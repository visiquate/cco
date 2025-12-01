//! Lifecycle hook executor
//!
//! Executes HTTP and bash hooks for lifecycle events (SessionStart, PreCompact, etc.)

use super::error::{HookError, HookResult};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, trace};

/// Execute a lifecycle hook (SessionStart, PreCompact, etc.)
///
/// # Arguments
///
/// * `hook_config` - Hook configuration from settings (type, url, method, etc.)
/// * `payload` - JSON payload to send to the hook
///
/// # Returns
///
/// JSON response from the hook execution
pub async fn execute_lifecycle_hook(hook_config: &Value, payload: Value) -> HookResult<Value> {
    let hook_type = hook_config["type"]
        .as_str()
        .ok_or_else(|| HookError::invalid_config("Hook type missing"))?;

    let timeout_ms = hook_config["timeout_ms"].as_u64().unwrap_or(5000);

    debug!(
        "Executing lifecycle hook: type={}, timeout={}ms",
        hook_type, timeout_ms
    );

    match hook_type {
        "http" => timeout(
            Duration::from_millis(timeout_ms),
            execute_http_hook(hook_config, payload),
        )
        .await
        .map_err(|_| {
            error!("HTTP hook timed out after {}ms", timeout_ms);
            HookError::timeout("lifecycle_hook", Duration::from_millis(timeout_ms))
        })?,
        "bash" => timeout(
            Duration::from_millis(timeout_ms),
            execute_bash_hook(hook_config, payload),
        )
        .await
        .map_err(|_| {
            error!("Bash hook timed out after {}ms", timeout_ms);
            HookError::timeout("lifecycle_hook", Duration::from_millis(timeout_ms))
        })?,
        _ => {
            error!("Unknown hook type: {}", hook_type);
            Err(HookError::invalid_config(&format!(
                "Unknown hook type: {}",
                hook_type
            )))
        }
    }
}

/// Execute an HTTP hook
async fn execute_http_hook(config: &Value, payload: Value) -> HookResult<Value> {
    let url = config["url"]
        .as_str()
        .ok_or_else(|| HookError::invalid_config("URL missing"))?;

    let method = config["method"].as_str().unwrap_or("POST");

    trace!("HTTP hook: {} {}", method, url);
    trace!("Payload: {}", serde_json::to_string_pretty(&payload)?);

    let client = reqwest::Client::new();
    let request = match method {
        "POST" => client.post(url).json(&payload),
        "GET" => client.get(url),
        _ => {
            return Err(HookError::invalid_config(&format!(
                "Unsupported method: {}",
                method
            )))
        }
    };

    let response = request
        .send()
        .await
        .map_err(|e| HookError::execution_failed("http_hook", e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        error!("HTTP hook failed: {} - {}", status, error_text);
        return Err(HookError::execution_failed(
            "http_hook",
            format!("{}: {}", status, error_text),
        ));
    }

    let result: Value = response
        .json()
        .await
        .map_err(|e| HookError::execution_failed("http_hook", e.to_string()))?;

    trace!(
        "HTTP hook response: {}",
        serde_json::to_string_pretty(&result)?
    );

    Ok(result)
}

/// Execute a bash hook
async fn execute_bash_hook(config: &Value, payload: Value) -> HookResult<Value> {
    let script = config["script"]
        .as_str()
        .ok_or_else(|| HookError::invalid_config("Script path missing"))?;

    let args: Vec<&str> = config["args"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_else(Vec::new);

    trace!("Bash hook: {} {:?}", script, args);
    trace!("Payload: {}", serde_json::to_string_pretty(&payload)?);

    let mut cmd = tokio::process::Command::new(script);
    cmd.args(&args);

    // Pass payload as environment variable
    cmd.env(
        "HOOK_PAYLOAD",
        serde_json::to_string(&payload).map_err(|e| {
            HookError::execution_failed("bash_hook", format!("Failed to serialize payload: {}", e))
        })?,
    );

    let output = cmd
        .output()
        .await
        .map_err(|e| HookError::execution_failed("bash_hook", e.to_string()))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        trace!("Bash hook output: {}", stdout);

        // Try to parse as JSON, fallback to plain output
        match serde_json::from_str::<Value>(&stdout) {
            Ok(json_value) => Ok(json_value),
            Err(_) => Ok(json!({"output": stdout.to_string()})),
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Bash hook failed: {}", stderr);
        Err(HookError::execution_failed("bash_hook", stderr.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_hook_missing_url() {
        let config = json!({
            "type": "http",
            "method": "POST"
        });

        let result = execute_lifecycle_hook(&config, json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bash_hook_missing_script() {
        let config = json!({
            "type": "bash"
        });

        let result = execute_lifecycle_hook(&config, json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unknown_hook_type() {
        let config = json!({
            "type": "unknown"
        });

        let result = execute_lifecycle_hook(&config, json!({})).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HookError::InvalidConfig { message: _ }
        ));
    }
}
