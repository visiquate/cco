//! Security middleware and utilities for CCO server
//!
//! Provides security hardening features:
//! - Localhost-only enforcement
//! - Connection tracking and limits
//! - Message size validation
//! - Security event logging

use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tracing::{debug, warn};

/// Connection tracking for rate limiting and connection limits
/// Uses DashMap for lock-free concurrent access (avoids double Arc issue)
#[derive(Clone)]
pub struct ConnectionTracker {
    /// Concurrent map of IP addresses to connection counts
    connections: Arc<DashMap<IpAddr, usize>>,
    /// Maximum connections per IP
    max_connections_per_ip: usize,
}

impl ConnectionTracker {
    /// Create a new connection tracker
    pub fn new(max_connections_per_ip: usize) -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            max_connections_per_ip,
        }
    }

    /// Try to acquire a connection slot for the given IP
    /// Returns true if allowed, false if limit exceeded
    pub async fn try_acquire(&self, ip: IpAddr) -> bool {
        // Check and increment atomically
        let mut entry = self.connections.entry(ip).or_insert(0);

        if *entry >= self.max_connections_per_ip {
            warn!(
                ip = %ip,
                current_connections = *entry,
                max_allowed = self.max_connections_per_ip,
                "Connection limit exceeded for IP"
            );
            false
        } else {
            *entry += 1;
            debug!(
                ip = %ip,
                current_connections = *entry,
                "Connection acquired"
            );
            true
        }
    }

    /// Release a connection slot for the given IP
    pub async fn release(&self, ip: IpAddr) {
        if let Some(mut entry) = self.connections.get_mut(&ip) {
            if *entry > 0 {
                *entry -= 1;
                debug!(
                    ip = %ip,
                    remaining_connections = *entry,
                    "Connection released"
                );
            }
            // Remove entry if count reaches zero to prevent unbounded growth
            if *entry == 0 {
                drop(entry);
                self.connections.remove(&ip);
            }
        }
    }

    /// Get current connection count for an IP
    pub async fn get_count(&self, ip: IpAddr) -> usize {
        self.connections.get(&ip).map(|r| *r).unwrap_or(0)
    }
}

/// Check if an IP address is localhost
pub fn is_localhost(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(addr) => addr.is_loopback(),
        IpAddr::V6(addr) => addr.is_loopback(),
    }
}

/// Middleware to enforce localhost-only access
pub async fn localhost_only_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let ip = addr.ip();

    if is_localhost(&ip) {
        debug!(
            ip = %ip,
            path = %request.uri().path(),
            "Localhost connection accepted"
        );
        next.run(request).await
    } else {
        warn!(
            ip = %ip,
            path = %request.uri().path(),
            "Remote connection blocked - localhost only"
        );
        (
            StatusCode::FORBIDDEN,
            "Access denied: localhost connections only",
        )
            .into_response()
    }
}

/// Validate WebSocket message size
///
/// # Arguments
/// * `data` - The message data to validate
/// * `max_size` - Maximum allowed message size in bytes
///
/// # Returns
/// * `Ok(())` if the message is within size limits
/// * `Err(String)` with error message if size is exceeded
pub fn validate_message_size(data: &[u8], max_size: usize) -> Result<(), String> {
    if data.len() > max_size {
        debug!(
            message_size = data.len(),
            max_size = max_size,
            "Message size limit exceeded"
        );
        Err(format!(
            "Message size {} exceeds maximum allowed size {}",
            data.len(),
            max_size
        ))
    } else {
        Ok(())
    }
}

/// Validate terminal resize dimensions
///
/// # Arguments
/// * `cols` - Number of columns
/// * `rows` - Number of rows
///
/// # Returns
/// * `Ok(())` if dimensions are valid
/// * `Err(String)` with error message if dimensions are invalid
pub fn validate_terminal_dimensions(cols: u16, rows: u16) -> Result<(), String> {
    const MIN_DIMENSION: u16 = 1;
    const MAX_DIMENSION: u16 = 1000;

    if cols < MIN_DIMENSION || cols > MAX_DIMENSION {
        return Err(format!(
            "Invalid columns {}: must be between {} and {}",
            cols, MIN_DIMENSION, MAX_DIMENSION
        ));
    }

    if rows < MIN_DIMENSION || rows > MAX_DIMENSION {
        return Err(format!(
            "Invalid rows {}: must be between {} and {}",
            rows, MIN_DIMENSION, MAX_DIMENSION
        ));
    }

    Ok(())
}

/// Validate UTF-8 encoding of message text
///
/// # Arguments
/// * `data` - The data to validate as UTF-8
///
/// # Returns
/// * `Ok(())` if data is valid UTF-8
/// * `Err(String)` with error message if encoding is invalid
pub fn validate_utf8(data: &[u8]) -> Result<(), String> {
    std::str::from_utf8(data)
        .map(|_| ())
        .map_err(|e| {
            debug!("Invalid UTF-8 encoding: {}", e);
            "Invalid UTF-8 encoding".to_string()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_localhost() {
        // IPv4 localhost
        let ipv4_localhost = "127.0.0.1".parse::<IpAddr>().unwrap();
        assert!(is_localhost(&ipv4_localhost));

        // IPv6 localhost
        let ipv6_localhost = "::1".parse::<IpAddr>().unwrap();
        assert!(is_localhost(&ipv6_localhost));

        // Non-localhost IPv4
        let ipv4_remote = "192.168.1.1".parse::<IpAddr>().unwrap();
        assert!(!is_localhost(&ipv4_remote));

        // Non-localhost IPv6
        let ipv6_remote = "2001:db8::1".parse::<IpAddr>().unwrap();
        assert!(!is_localhost(&ipv6_remote));
    }

    #[tokio::test]
    async fn test_connection_tracker() {
        let tracker = ConnectionTracker::new(2);
        let ip = "127.0.0.1".parse::<IpAddr>().unwrap();

        // Should allow first connection
        assert!(tracker.try_acquire(ip).await);
        // DashMap operations are instant, no await needed for get_count but we keep async for API consistency
        assert_eq!(tracker.get_count(ip).await, 1);

        // Should allow second connection
        assert!(tracker.try_acquire(ip).await);
        assert_eq!(tracker.get_count(ip).await, 2);

        // Should reject third connection
        assert!(!tracker.try_acquire(ip).await);
        assert_eq!(tracker.get_count(ip).await, 2);

        // Release one connection
        tracker.release(ip).await;
        assert_eq!(tracker.get_count(ip).await, 1);

        // Should allow connection again
        assert!(tracker.try_acquire(ip).await);
        assert_eq!(tracker.get_count(ip).await, 2);
    }

    #[test]
    fn test_validate_message_size() {
        let small_data = vec![0u8; 100];
        assert!(validate_message_size(&small_data, 1024).is_ok());

        let large_data = vec![0u8; 2048];
        assert!(validate_message_size(&large_data, 1024).is_err());
    }

    #[test]
    fn test_validate_terminal_dimensions() {
        // Valid dimensions
        assert!(validate_terminal_dimensions(80, 24).is_ok());
        assert!(validate_terminal_dimensions(1, 1).is_ok());
        assert!(validate_terminal_dimensions(1000, 1000).is_ok());

        // Invalid dimensions - too small
        assert!(validate_terminal_dimensions(0, 24).is_err());
        assert!(validate_terminal_dimensions(80, 0).is_err());

        // Invalid dimensions - too large
        assert!(validate_terminal_dimensions(1001, 24).is_err());
        assert!(validate_terminal_dimensions(80, 1001).is_err());
    }

    #[test]
    fn test_validate_utf8() {
        // Valid UTF-8
        let valid_utf8 = "Hello, world!".as_bytes();
        assert!(validate_utf8(valid_utf8).is_ok());

        // Invalid UTF-8
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        assert!(validate_utf8(&invalid_utf8).is_err());
    }
}
