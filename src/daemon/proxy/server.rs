//! HTTP proxy server for routing Claude API requests
//!
//! Intercepts HTTP/HTTPS traffic to api.anthropic.com and routes based on agent type:
//! - Azure agents → Azure GPT-5.1-codex-mini
//! - Others → Claude (pass-through)

use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

use super::router::should_route_to_azure;
use super::translator::{extract_agent_type_from_body, translate_request_to_azure, translate_response_from_azure};
use crate::daemon::security::azure_credential;

/// Azure OpenAI endpoint configuration
const AZURE_ENDPOINT: &str = "https://cco-resource.cognitiveservices.azure.com";
const AZURE_DEPLOYMENT: &str = "gpt-5.1-codex-mini";
const AZURE_API_VERSION: &str = "2024-05-01-preview";

/// Anthropic endpoint
const ANTHROPIC_ENDPOINT: &str = "https://api.anthropic.com";

/// Start the HTTP proxy server
///
/// Listens on the provided address and intercepts requests to api.anthropic.com,
/// routing them based on agent type classification.
///
/// # Arguments
/// * `addr` - Socket address to listen on (e.g., "127.0.0.1:8888")
///
/// # Returns
/// Result containing the actual port bound (useful when port 0 is specified)
pub async fn start_proxy_server(addr: &str) -> Result<u16> {
    let socket_addr: SocketAddr = addr
        .parse()
        .context("Failed to parse proxy server address")?;

    let listener = TcpListener::bind(&socket_addr)
        .await
        .context("Failed to bind proxy server")?;

    let actual_addr = listener.local_addr()?;
    let actual_port = actual_addr.port();

    info!(
        "🚀 HTTP proxy server started on {}",
        actual_addr
    );
    info!(
        "   Claude API: {} → pass-through",
        ANTHROPIC_ENDPOINT
    );
    info!(
        "   Azure API: {} → routing selected agents",
        AZURE_ENDPOINT
    );

    // Create a shared HTTP client with TLS support for upstream requests
    let http_client = Arc::new(
        Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout for LLM requests
            .build()
            .context("Failed to create HTTP client")?
    );

    // Spawn the proxy accept loop
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, peer_addr)) => {
                    debug!("Accepted connection from {}", peer_addr);
                    let client = Arc::clone(&http_client);
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(socket, client).await {
                            warn!("Error handling connection from {}: {}", peer_addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    });

    Ok(actual_port)
}

/// Handle a single proxy connection
///
/// Reads the HTTP request, determines routing, and proxies to appropriate backend.
async fn handle_connection(mut client_socket: TcpStream, http_client: Arc<Client>) -> Result<()> {
    // Read request from client
    let request_data = read_http_request(&mut client_socket).await?;

    debug!(
        request_size = request_data.len(),
        "Received request from client"
    );

    // Parse HTTP request
    let (method, path, headers, body) = parse_http_request(&request_data)?;

    debug!(
        method = &method,
        path = &path,
        body_size = body.len(),
        "Parsed HTTP request"
    );

    // Determine routing destination
    let agent_type = extract_agent_type_from_body(&body);
    let route_to_azure = agent_type
        .as_ref()
        .map(|at| should_route_to_azure(at))
        .unwrap_or(false);

    if route_to_azure {
        info!(
            agent_type = agent_type.clone().unwrap_or_default(),
            "Routing request to Azure"
        );
        handle_azure_request(&mut client_socket, &http_client, &method, &path, &headers, &body).await?;
    } else {
        info!("Routing request to Anthropic (pass-through)");
        handle_anthropic_passthrough(&mut client_socket, &http_client, &method, &path, &headers, &body).await?;
    }

    Ok(())
}

/// Read HTTP request from client socket
///
/// Reads until we have complete headers and body (if present).
async fn read_http_request(socket: &mut TcpStream) -> Result<Vec<u8>> {
    use tokio::io::AsyncReadExt;

    let mut buffer = vec![0u8; 65536]; // 64KB initial buffer
    let mut request_data = Vec::new();

    match socket.read(&mut buffer).await {
        Ok(0) => return Err(anyhow!("Client closed connection before sending request")),
        Ok(n) => {
            request_data.extend_from_slice(&buffer[..n]);
        }
        Err(e) => return Err(anyhow!("Failed to read from client: {}", e)),
    }

    debug!(
        received_bytes = request_data.len(),
        "Read HTTP request"
    );

    Ok(request_data)
}

/// Parse HTTP request into components
///
/// Extracts method, path, headers, and body from raw HTTP data.
fn parse_http_request(data: &[u8]) -> Result<(String, String, Vec<(String, String)>, Vec<u8>)> {
    let text = String::from_utf8_lossy(data);
    let parts: Vec<&str> = text.split("\r\n\r\n").collect();

    if parts.is_empty() {
        return Err(anyhow!("Invalid HTTP request: no headers"));
    }

    let header_section = parts[0];
    let body = if parts.len() > 1 {
        parts[1].as_bytes().to_vec()
    } else {
        Vec::new()
    };

    // Parse request line
    let lines: Vec<&str> = header_section.split("\r\n").collect();
    if lines.is_empty() {
        return Err(anyhow!("Empty HTTP request"));
    }

    let request_line: Vec<&str> = lines[0].split_whitespace().collect();
    if request_line.len() < 2 {
        return Err(anyhow!("Invalid request line: {}", lines[0]));
    }

    let method = request_line[0].to_string();
    let path = request_line[1].to_string();

    // Parse headers
    let mut headers = Vec::new();
    for line in &lines[1..] {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.push((key.trim().to_string(), value.trim().to_string()));
        }
    }

    debug!(
        method = &method,
        path = &path,
        headers_count = headers.len(),
        body_size = body.len(),
        "Parsed HTTP request"
    );

    Ok((method, path, headers, body))
}

/// Route request to Anthropic (pass-through)
///
/// Forwards the request to Anthropic API without modification using HTTPS.
async fn handle_anthropic_passthrough(
    client_socket: &mut TcpStream,
    http_client: &Client,
    method: &str,
    proxy_path: &str,
    headers: &[(String, String)],
    body: &[u8],
) -> Result<()> {
    use tokio::io::AsyncWriteExt;

    info!(
        method = method,
        path = proxy_path,
        "Forwarding request to Anthropic via HTTPS"
    );

    // Build the full URL
    let url = format!("{}{}", ANTHROPIC_ENDPOINT, proxy_path);

    // Build the request
    let mut request_builder = match method {
        "GET" => http_client.get(&url),
        "POST" => http_client.post(&url),
        "PUT" => http_client.put(&url),
        "DELETE" => http_client.delete(&url),
        "PATCH" => http_client.patch(&url),
        _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
    };

    // Add headers (skip Host and Content-Length as reqwest handles these)
    for (key, value) in headers {
        let key_lower = key.to_lowercase();
        if key_lower != "host" && key_lower != "content-length" && key_lower != "connection" {
            request_builder = request_builder.header(key, value);
        }
    }

    // Add body if present
    if !body.is_empty() {
        request_builder = request_builder.body(body.to_vec());
    }

    // Send request and get response
    let response = request_builder
        .send()
        .await
        .context("Failed to send request to Anthropic")?;

    let status = response.status();
    let response_headers = response.headers().clone();
    let response_body = response
        .bytes()
        .await
        .context("Failed to read response body from Anthropic")?;

    debug!(
        status = %status,
        response_size = response_body.len(),
        "Received response from Anthropic"
    );

    // Build HTTP response for client
    let mut http_response = format!("HTTP/1.1 {} {}\r\n", status.as_u16(), status.canonical_reason().unwrap_or("OK"));

    // Forward relevant headers
    for (key, value) in response_headers.iter() {
        if let Ok(v) = value.to_str() {
            http_response.push_str(&format!("{}: {}\r\n", key, v));
        }
    }
    http_response.push_str("\r\n");

    // Send response to client
    client_socket.write_all(http_response.as_bytes()).await
        .context("Failed to write response headers to client")?;
    client_socket.write_all(&response_body).await
        .context("Failed to write response body to client")?;

    debug!(
        response_size = response_body.len(),
        "Forwarded response from Anthropic"
    );

    Ok(())
}

/// Route request to Azure
///
/// Translates request to Azure format, sends to Azure via HTTPS, and translates response back.
async fn handle_azure_request(
    client_socket: &mut TcpStream,
    http_client: &Client,
    _method: &str,
    _path: &str,
    _headers: &[(String, String)],
    body: &[u8],
) -> Result<()> {
    use tokio::io::AsyncWriteExt;

    // Get Azure API key
    let azure_key = azure_credential::get_azure_api_key()
        .ok_or_else(|| anyhow!("Azure API key not available"))?;

    info!("Translating request to Azure format");

    // Parse request body as JSON
    let request_json: serde_json::Value = serde_json::from_slice(body)
        .context("Failed to parse request body as JSON")?;

    // Translate to Azure format
    let azure_request = translate_request_to_azure(&request_json)
        .context("Failed to translate request to Azure format")?;

    // Build Azure endpoint URL
    let azure_url = format!(
        "{}/openai/deployments/{}/chat/completions?api-version={}",
        AZURE_ENDPOINT, AZURE_DEPLOYMENT, AZURE_API_VERSION
    );

    // Send request to Azure via HTTPS
    let response = http_client
        .post(&azure_url)
        .header("api-key", &azure_key)
        .header("Content-Type", "application/json")
        .json(&azure_request)
        .send()
        .await
        .context("Failed to send request to Azure")?;

    let status = response.status();
    let response_body = response
        .bytes()
        .await
        .context("Failed to read response body from Azure")?;

    debug!(
        status = %status,
        response_size = response_body.len(),
        "Received response from Azure"
    );

    // Parse Azure response
    let azure_response: serde_json::Value = serde_json::from_slice(&response_body)
        .context("Failed to parse Azure response")?;

    // Translate back to Anthropic format
    let anthropic_response = translate_response_from_azure(&azure_response)
        .context("Failed to translate response from Azure format")?;

    let final_body = serde_json::to_vec(&anthropic_response)
        .context("Failed to serialize Anthropic response")?;

    // Build HTTP response for client
    let http_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        final_body.len()
    );

    // Send response to client
    client_socket.write_all(http_response.as_bytes()).await
        .context("Failed to write response headers to client")?;
    client_socket.write_all(&final_body).await
        .context("Failed to write response body to client")?;

    debug!(
        response_size = final_body.len(),
        "Forwarded translated response to client"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Format HTTP request (test helper)
    fn format_http_request(method: &str, path: &str, headers: &[(String, String)], _body: &[u8]) -> Vec<u8> {
        let mut request = format!("{} {} HTTP/1.1\r\n", method, path);

        for (key, value) in headers {
            request.push_str(&format!("{}: {}\r\n", key, value));
        }

        request.push_str("Connection: close\r\n");
        request.push_str("\r\n");

        request.into_bytes()
    }

    #[test]
    fn test_format_http_request() {
        let headers = vec![
            ("Host".to_string(), "api.anthropic.com".to_string()),
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        let request = format_http_request("POST", "/v1/messages", &headers, b"");
        let request_str = String::from_utf8(request).unwrap();

        assert!(request_str.contains("POST /v1/messages HTTP/1.1"));
        assert!(request_str.contains("Host: api.anthropic.com"));
        assert!(request_str.contains("Content-Type: application/json"));
    }

    #[test]
    fn test_parse_http_request() {
        let request = b"POST /api/test HTTP/1.1\r\nHost: example.com\r\nContent-Length: 5\r\n\r\nhello";
        let (method, path, headers, body) = parse_http_request(request).unwrap();

        assert_eq!(method, "POST");
        assert_eq!(path, "/api/test");
        assert!(!headers.is_empty());
        assert_eq!(body, b"hello");
    }

    #[test]
    fn test_parse_http_request_no_body() {
        let request = b"GET /api/test HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let (method, path, _, body) = parse_http_request(request).unwrap();

        assert_eq!(method, "GET");
        assert_eq!(path, "/api/test");
        assert!(body.is_empty());
    }
}
