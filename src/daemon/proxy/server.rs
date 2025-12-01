//! HTTP proxy server for routing Claude API requests
//!
//! Intercepts HTTP/HTTPS traffic to api.anthropic.com and routes based on agent type:
//! - Azure agents → Azure GPT-5.1-codex-mini
//! - Others → Claude (pass-through)

use anyhow::{anyhow, Context, Result};
use std::net::SocketAddr;
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

    // Spawn the proxy accept loop
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, peer_addr)) => {
                    debug!("Accepted connection from {}", peer_addr);
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(socket).await {
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
async fn handle_connection(mut client: TcpStream) -> Result<()> {
    // Read request from client
    let request_data = read_http_request(&mut client).await?;

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
        handle_azure_request(&mut client, &method, &path, &headers, &body).await?;
    } else {
        info!("Routing request to Anthropic (pass-through)");
        handle_anthropic_passthrough(&mut client, &method, &path, &headers, &body).await?;
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
/// Forwards the request to Anthropic API without modification.
async fn handle_anthropic_passthrough(
    client: &mut TcpStream,
    method: &str,
    proxy_path: &str,
    headers: &[(String, String)],
    body: &[u8],
) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    info!(
        method = method,
        "Forwarding request to Anthropic"
    );

    // Connect to Anthropic
    let remote = format!("api.anthropic.com:443");
    let mut remote_socket = tokio::net::TcpStream::connect(&remote)
        .await
        .context("Failed to connect to Anthropic")?;

    // Build and send request to Anthropic
    let request = format_http_request(method, proxy_path, headers, body);
    remote_socket.write_all(&request).await
        .context("Failed to write to Anthropic")?;
    remote_socket.write_all(body).await
        .context("Failed to write body to Anthropic")?;

    // Read response from Anthropic
    let mut response = Vec::new();
    let mut buffer = vec![0u8; 65536];
    loop {
        match remote_socket.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => response.extend_from_slice(&buffer[..n]),
            Err(e) => {
                warn!("Error reading from Anthropic: {}", e);
                break;
            }
        }
    }

    // Forward response to client
    client.write_all(&response).await
        .context("Failed to write response to client")?;

    debug!(
        response_size = response.len(),
        "Forwarded response from Anthropic"
    );

    Ok(())
}

/// Route request to Azure
///
/// Translates request to Azure format, sends to Azure, and translates response back.
async fn handle_azure_request(
    client: &mut TcpStream,
    method: &str,
    _path: &str,
    _headers: &[(String, String)],
    body: &[u8],
) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

    let azure_body = serde_json::to_vec(&azure_request)
        .context("Failed to serialize Azure request")?;

    // Build Azure endpoint URL
    let azure_url = format!(
        "/openai/deployments/{}/chat/completions?api-version={}",
        AZURE_DEPLOYMENT, AZURE_API_VERSION
    );

    // Connect to Azure
    let remote = format!("cco-resource.cognitiveservices.azure.com:443");
    let mut remote_socket = tokio::net::TcpStream::connect(&remote)
        .await
        .context("Failed to connect to Azure")?;

    // Build request to Azure with API key header
    let azure_headers = vec![
        ("Host".to_string(), "cco-resource.cognitiveservices.azure.com".to_string()),
        ("api-key".to_string(), azure_key),
        ("Content-Type".to_string(), "application/json".to_string()),
        ("Content-Length".to_string(), azure_body.len().to_string()),
    ];

    let request = format_http_request(method, &azure_url, &azure_headers, &azure_body);
    remote_socket.write_all(&request).await
        .context("Failed to write to Azure")?;
    remote_socket.write_all(&azure_body).await
        .context("Failed to write body to Azure")?;

    // Read response from Azure
    let mut response_body = Vec::new();
    let mut buffer = vec![0u8; 65536];
    loop {
        match remote_socket.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => response_body.extend_from_slice(&buffer[..n]),
            Err(e) => {
                warn!("Error reading from Azure: {}", e);
                break;
            }
        }
    }

    // Extract headers and body from response
    let response_str = String::from_utf8_lossy(&response_body);
    let response_body_part = if let Some(pos) = response_str.find("\r\n\r\n") {
        let (_, body) = response_str.split_at(pos + 4);
        body.as_bytes()
    } else {
        response_body.as_slice()
    };

    // Parse Azure response
    let azure_response: serde_json::Value = serde_json::from_slice(response_body_part)
        .context("Failed to parse Azure response")?;

    // Translate back to Anthropic format
    let anthropic_response = translate_response_from_azure(&azure_response)
        .context("Failed to translate response from Azure format")?;

    let final_body = serde_json::to_vec(&anthropic_response)
        .context("Failed to serialize Anthropic response")?;

    // Build response headers for client
    let response_headers = vec![
        ("Content-Type".to_string(), "application/json".to_string()),
        ("Content-Length".to_string(), final_body.len().to_string()),
        ("Connection".to_string(), "close".to_string()),
    ];

    // Build and send response to client
    let response = format_http_response(&response_headers, &final_body);
    client.write_all(&response).await
        .context("Failed to write response to client")?;

    debug!(
        response_size = final_body.len(),
        "Forwarded translated response to client"
    );

    Ok(())
}

/// Format HTTP request
fn format_http_request(method: &str, path: &str, headers: &[(String, String)], body: &[u8]) -> Vec<u8> {
    let mut request = format!("{} {} HTTP/1.1\r\n", method, path);

    for (key, value) in headers {
        request.push_str(&format!("{}: {}\r\n", key, value));
    }

    request.push_str("Connection: close\r\n");
    request.push_str("\r\n");

    let mut result = request.into_bytes();
    result.extend_from_slice(body);
    result
}

/// Format HTTP response
fn format_http_response(headers: &[(String, String)], body: &[u8]) -> Vec<u8> {
    let mut response = "HTTP/1.1 200 OK\r\n".to_string();

    for (key, value) in headers {
        response.push_str(&format!("{}: {}\r\n", key, value));
    }

    response.push_str("\r\n");

    let mut result = response.into_bytes();
    result.extend_from_slice(body);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

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
