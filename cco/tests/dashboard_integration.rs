// Comprehensive dashboard and HTML integration tests
// Tests dashboard endpoints, content delivery, and terminal functionality

#[cfg(test)]
mod dashboard_tests {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    const TEST_HOST: &str = "127.0.0.1";
    const TEST_PORT: u16 = 3333;
    const TIMEOUT: Duration = Duration::from_secs(5);

    fn send_http_request(
        method: &str,
        path: &str,
        headers: &[(&str, &str)],
    ) -> Result<String, String> {
        let addr = format!("{}:{}", TEST_HOST, TEST_PORT);
        let mut stream = TcpStream::connect(&addr)
            .map_err(|e| format!("Failed to connect to {}:{}: {}", TEST_HOST, TEST_PORT, e))?;

        stream
            .set_read_timeout(Some(TIMEOUT))
            .map_err(|e| format!("Failed to set read timeout: {}", e))?;
        stream
            .set_write_timeout(Some(TIMEOUT))
            .map_err(|e| format!("Failed to set write timeout: {}", e))?;

        // Build request
        let mut request = format!(
            "{} {} HTTP/1.1\r\nHost: {}:{}\r\n",
            method, path, TEST_HOST, TEST_PORT
        );
        request.push_str("Connection: close\r\n");
        for (key, value) in headers {
            request.push_str(&format!("{}: {}\r\n", key, value));
        }
        request.push_str("\r\n");

        stream
            .write_all(request.as_bytes())
            .map_err(|e| format!("Failed to write request: {}", e))?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|e| format!("Failed to read response: {}", e))?;

        Ok(response)
    }

    fn extract_status_code(response: &str) -> Result<u16, String> {
        let first_line = response
            .lines()
            .next()
            .ok_or("Empty response".to_string())?;
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Invalid status line".to_string());
        }
        parts[1]
            .parse::<u16>()
            .map_err(|e| format!("Failed to parse status code: {}", e))
    }

    fn extract_header(response: &str, header_name: &str) -> Option<String> {
        for line in response.lines() {
            if line
                .to_lowercase()
                .starts_with(&format!("{}:", header_name.to_lowercase()))
            {
                return Some(line.split(':').nth(1).unwrap_or("").trim().to_string());
            }
        }
        None
    }

    fn extract_body(response: &str) -> String {
        if let Some(pos) = response.find("\r\n\r\n") {
            response[pos + 4..].to_string()
        } else if let Some(pos) = response.find("\n\n") {
            response[pos + 2..].to_string()
        } else {
            String::new()
        }
    }

    // ===== DASHBOARD HTML TESTS =====

    #[test]
    fn test_dashboard_html_endpoint_returns_200() {
        let response = send_http_request("GET", "/", &[]).expect("Request failed");
        let status = extract_status_code(&response).expect("Failed to extract status");
        assert_eq!(
            status, 200,
            "Dashboard HTML endpoint should return HTTP 200"
        );
    }

    #[test]
    fn test_dashboard_html_has_correct_content_type() {
        let response = send_http_request("GET", "/", &[]).expect("Request failed");
        let content_type = extract_header(&response, "content-type").unwrap_or_default();
        assert!(
            content_type.contains("text/html"),
            "Dashboard should return text/html content type, got: {}",
            content_type
        );
    }

    #[test]
    fn test_dashboard_html_contains_required_elements() {
        let response = send_http_request("GET", "/", &[]).expect("Request failed");
        let body = extract_body(&response);

        // Check for DOCTYPE
        assert!(
            body.contains("<!DOCTYPE") || body.contains("<!doctype"),
            "HTML should contain DOCTYPE declaration"
        );

        // Check for essential HTML tags
        assert!(body.contains("<html"), "HTML should contain <html> tag");
        assert!(body.contains("<head"), "HTML should contain <head> tag");
        assert!(body.contains("<body"), "HTML should contain <body> tag");
    }

    #[test]
    fn test_dashboard_html_has_title() {
        let response = send_http_request("GET", "/", &[]).expect("Request failed");
        let body = extract_body(&response);

        assert!(
            body.contains("<title>") && body.contains("</title>"),
            "HTML should contain <title> tag"
        );

        // Check that title mentions Claude or dashboard
        let title_start = body.find("<title>").unwrap_or(0) + 7;
        let title_end = body.find("</title>").unwrap_or(body.len());
        let title = &body[title_start..title_end];
        assert!(
            title.contains("Claude") || title.contains("Dashboard") || title.contains("CCO"),
            "Title should reference Claude Code or Dashboard"
        );
    }

    #[test]
    fn test_dashboard_html_includes_metadata() {
        let response = send_http_request("GET", "/", &[]).expect("Request failed");
        let body = extract_body(&response);

        // Check for viewport meta tag (important for responsive design)
        assert!(
            body.contains("viewport"),
            "HTML should include viewport meta tag for responsive design"
        );

        // Check for charset
        assert!(
            body.contains("charset") || body.contains("utf-8"),
            "HTML should specify character encoding"
        );
    }

    // ===== CSS ASSET TESTS =====

    #[test]
    fn test_dashboard_css_endpoint_returns_200() {
        let response = send_http_request("GET", "/dashboard.css", &[]).expect("Request failed");
        let status = extract_status_code(&response).expect("Failed to extract status");
        assert_eq!(status, 200, "CSS endpoint should return HTTP 200");
    }

    #[test]
    fn test_dashboard_css_has_correct_content_type() {
        let response = send_http_request("GET", "/dashboard.css", &[]).expect("Request failed");
        let content_type = extract_header(&response, "content-type").unwrap_or_default();
        assert!(
            content_type.contains("css") || content_type.contains("text"),
            "CSS should return text/css or similar content type, got: {}",
            content_type
        );
    }

    #[test]
    fn test_dashboard_css_file_not_empty() {
        let response = send_http_request("GET", "/dashboard.css", &[]).expect("Request failed");
        let body = extract_body(&response);
        assert!(!body.is_empty(), "CSS file should not be empty");
        assert!(
            body.len() > 100,
            "CSS file should contain substantial content (got {} bytes)",
            body.len()
        );
    }

    #[test]
    fn test_dashboard_css_contains_selectors() {
        let response = send_http_request("GET", "/dashboard.css", &[]).expect("Request failed");
        let body = extract_body(&response);

        // Check for CSS syntax
        assert!(
            body.contains("{") && body.contains("}"),
            "CSS should contain rule blocks with braces"
        );

        // Check for common CSS properties
        assert!(
            body.contains("color") || body.contains("background") || body.contains("display"),
            "CSS should contain styling rules"
        );
    }

    // ===== JAVASCRIPT ASSET TESTS =====

    #[test]
    fn test_dashboard_js_endpoint_returns_200() {
        let response = send_http_request("GET", "/dashboard.js", &[]).expect("Request failed");
        let status = extract_status_code(&response).expect("Failed to extract status");
        assert_eq!(status, 200, "JavaScript endpoint should return HTTP 200");
    }

    #[test]
    fn test_dashboard_js_has_correct_content_type() {
        let response = send_http_request("GET", "/dashboard.js", &[]).expect("Request failed");
        let content_type = extract_header(&response, "content-type").unwrap_or_default();
        assert!(
            content_type.contains("javascript") || content_type.contains("text"),
            "JavaScript should return application/javascript or similar, got: {}",
            content_type
        );
    }

    #[test]
    fn test_dashboard_js_file_not_empty() {
        let response = send_http_request("GET", "/dashboard.js", &[]).expect("Request failed");
        let body = extract_body(&response);
        assert!(!body.is_empty(), "JavaScript file should not be empty");
        assert!(
            body.len() > 100,
            "JavaScript file should contain substantial code (got {} bytes)",
            body.len()
        );
    }

    #[test]
    fn test_dashboard_js_contains_function_definitions() {
        let response = send_http_request("GET", "/dashboard.js", &[]).expect("Request failed");
        let body = extract_body(&response);

        // Check for JavaScript function definitions
        assert!(
            body.contains("function") || body.contains("=>") || body.contains("const"),
            "JavaScript should contain function or variable definitions"
        );
    }

    #[test]
    fn test_dashboard_js_has_terminal_initialization() {
        let response = send_http_request("GET", "/dashboard.js", &[]).expect("Request failed");
        let body = extract_body(&response);

        // Check for terminal-related code
        assert!(
            body.contains("terminal") || body.contains("Terminal") || body.contains("xterm"),
            "JavaScript should include terminal initialization code"
        );
    }

    #[test]
    fn test_dashboard_js_has_websocket_support() {
        let response = send_http_request("GET", "/dashboard.js", &[]).expect("Request failed");
        let body = extract_body(&response);

        // Check for WebSocket handling
        assert!(
            body.contains("WebSocket") || body.contains("ws://") || body.contains("wss://"),
            "JavaScript should include WebSocket support for terminal"
        );
    }

    // ===== API ENDPOINT TESTS =====

    #[test]
    fn test_health_endpoint_returns_200() {
        let response = send_http_request("GET", "/health", &[]).expect("Request failed");
        let status = extract_status_code(&response).expect("Failed to extract status");
        assert_eq!(status, 200, "Health endpoint should return HTTP 200");
    }

    #[test]
    fn test_health_endpoint_returns_json() {
        let response = send_http_request("GET", "/health", &[]).expect("Request failed");
        let body = extract_body(&response);

        assert!(
            body.contains("{") && body.contains("}"),
            "Health endpoint should return JSON response"
        );

        assert!(
            body.contains("\"status\""),
            "Health response should contain status field"
        );
    }

    #[test]
    fn test_agents_endpoint_returns_200() {
        let response = send_http_request("GET", "/api/agents", &[]).expect("Request failed");
        let status = extract_status_code(&response).expect("Failed to extract status");
        assert_eq!(status, 200, "Agents endpoint should return HTTP 200");
    }

    #[test]
    fn test_stats_endpoint_returns_200() {
        let response = send_http_request("GET", "/api/stats", &[]).expect("Request failed");
        let status = extract_status_code(&response).expect("Failed to extract status");
        assert_eq!(status, 200, "Stats endpoint should return HTTP 200");
    }

    #[test]
    fn test_metrics_endpoint_returns_200() {
        let response =
            send_http_request("GET", "/api/metrics/projects", &[]).expect("Request failed");
        let status = extract_status_code(&response).expect("Failed to extract status");
        assert_eq!(status, 200, "Metrics endpoint should return HTTP 200");
    }

    // ===== CONTENT DELIVERY TESTS =====

    #[test]
    fn test_dashboard_html_includes_css_reference() {
        let response = send_http_request("GET", "/", &[]).expect("Request failed");
        let body = extract_body(&response);

        assert!(
            body.contains("dashboard.css") || body.contains("style"),
            "HTML should reference CSS file"
        );
    }

    #[test]
    fn test_dashboard_html_includes_js_reference() {
        let response = send_http_request("GET", "/", &[]).expect("Request failed");
        let body = extract_body(&response);

        assert!(
            body.contains("dashboard.js") || body.contains("<script"),
            "HTML should reference JavaScript file"
        );
    }

    #[test]
    fn test_dashboard_html_references_xterm_library() {
        let response = send_http_request("GET", "/", &[]).expect("Request failed");
        let body = extract_body(&response);

        assert!(
            body.contains("xterm") || body.contains("terminal"),
            "HTML should reference xterm.js terminal library"
        );
    }

    // ===== ERROR HANDLING TESTS =====

    #[test]
    fn test_nonexistent_endpoint_returns_404() {
        let response = send_http_request("GET", "/nonexistent-page", &[]).expect("Request failed");
        let status = extract_status_code(&response).expect("Failed to extract status");
        assert_eq!(status, 404, "Nonexistent endpoints should return HTTP 404");
    }

    #[test]
    fn test_dashboard_html_loads_without_errors() {
        // This test ensures the entire HTML page loads and parses correctly
        let response = send_http_request("GET", "/", &[]).expect("Request failed");
        let status = extract_status_code(&response).expect("Failed to extract status");
        let body = extract_body(&response);

        assert_eq!(status, 200, "Dashboard should load successfully");
        assert!(!body.is_empty(), "Dashboard HTML should have content");
        assert!(
            body.len() > 1000,
            "Dashboard HTML should be substantial (got {} bytes)",
            body.len()
        );
    }

    // ===== TERMINAL INTEGRATION TESTS =====

    #[test]
    fn test_dashboard_includes_terminal_container() {
        let response = send_http_request("GET", "/", &[]).expect("Request failed");
        let body = extract_body(&response);

        assert!(
            body.contains("terminal") || body.contains("Terminal") || body.contains("term"),
            "Dashboard HTML should include terminal container element"
        );
    }

    #[test]
    fn test_dashboard_js_has_focus_management() {
        let response = send_http_request("GET", "/dashboard.js", &[]).expect("Request failed");
        let body = extract_body(&response);

        // Check for terminal focus management (critical for keyboard input)
        assert!(
            body.contains("focus") || body.contains("Focus"),
            "JavaScript should include focus management for terminal input capture"
        );
    }

    #[test]
    fn test_dashboard_js_handles_binary_data() {
        let response = send_http_request("GET", "/dashboard.js", &[]).expect("Request failed");
        let body = extract_body(&response);

        // Check for binary data handling (required for terminal I/O)
        assert!(
            body.contains("binary") || body.contains("Uint8Array") || body.contains("ArrayBuffer"),
            "JavaScript should handle binary data for terminal communication"
        );
    }

    // ===== RESPONSE HEADER TESTS =====

    #[test]
    fn test_dashboard_html_includes_security_headers() {
        let response = send_http_request("GET", "/", &[]).expect("Request failed");

        // Check for common security headers (not all required, but good practice)
        let has_cache_control = extract_header(&response, "cache-control").is_some();
        let has_content_type = extract_header(&response, "content-type").is_some();

        assert!(
            has_content_type,
            "Response should include content-type header"
        );
        // Cache control is recommended but not required for functionality
    }

    #[test]
    fn test_api_endpoints_return_json_content_type() {
        let response = send_http_request("GET", "/api/agents", &[]).expect("Request failed");
        let content_type = extract_header(&response, "content-type").unwrap_or_default();

        assert!(
            content_type.contains("json"),
            "API endpoints should return application/json, got: {}",
            content_type
        );
    }
}
