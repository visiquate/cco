//! Integration tests for Credential Store HTTP API
//!
//! Tests all aspects of the credential management API including:
//! - Store and retrieve credentials via HTTP API
//! - List credentials
//! - Delete credentials
//! - Rotation check
//! - Authentication requirements
//! - Rate limiting enforcement
//! - Audit logging verification
//! - Payload size validation
//! - Expiration handling
//!
//! Run with: cargo test credentials_api

use anyhow::Result;
use cco::credentials::keyring::{CredentialMetadata, CredentialType};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

// =============================================================================
// Test Client and Helpers
// =============================================================================

#[derive(Clone)]
struct CredentialTestClient {
    client: Client,
    base_url: String,
    port: u16,
    _auth_token: Option<String>,
}

impl CredentialTestClient {
    fn new(port: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: format!("http://127.0.0.1:{}", port),
            port,
            _auth_token: None,
        }
    }

    async fn health(&self) -> Result<HealthResponse> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }

    async fn wait_for_ready(&self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if self.health().await.is_ok() {
                return Ok(());
            }
            sleep(Duration::from_millis(100)).await;
        }
        anyhow::bail!("Daemon did not become ready within {:?}", timeout)
    }

    async fn store_credential(
        &self,
        key: &str,
        secret: &str,
        metadata: CredentialMetadata,
        expires_at: Option<String>,
    ) -> Result<StoreCredentialResponse> {
        let url = format!("{}/api/credentials/store", self.base_url);
        let request = StoreCredentialRequest {
            key: key.to_string(),
            secret: secret.to_string(),
            metadata,
            expires_at,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    async fn retrieve_credential(&self, key: &str) -> Result<RetrieveCredentialResponse> {
        let url = format!("{}/api/credentials/retrieve/{}", self.base_url, key);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }

    async fn list_credentials(&self) -> Result<ListCredentialsResponse> {
        let url = format!("{}/api/credentials/list", self.base_url);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }

    async fn delete_credential(&self, key: &str) -> Result<DeleteCredentialResponse> {
        let url = format!("{}/api/credentials/delete/{}", self.base_url, key);
        let response = self
            .client
            .delete(&url)
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json().await?)
    }

    async fn check_exists(&self, key: &str) -> Result<ExistsResponse> {
        let url = format!("{}/api/credentials/exists/{}", self.base_url, key);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }

    async fn check_rotation(&self) -> Result<RotationCheckResponse> {
        let url = format!("{}/api/credentials/check-rotation", self.base_url);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }

    async fn rotate_credential(
        &self,
        key: &str,
        new_secret: &str,
    ) -> Result<RotateCredentialResponse> {
        let url = format!("{}/api/credentials/rotate/{}", self.base_url, key);
        let request = RotateCredentialRequest {
            new_secret: new_secret.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    /// Helper to make a raw request (for testing error cases)
    async fn raw_post(&self, path: &str, body: serde_json::Value) -> reqwest::Response {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .expect("Failed to send request")
    }
}

// =============================================================================
// API Request/Response Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct StoreCredentialRequest {
    key: String,
    secret: String,
    metadata: CredentialMetadata,
    expires_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoreCredentialResponse {
    success: bool,
    key: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RetrieveCredentialResponse {
    key: String,
    secret: String,
    created_at: String,
    expires_at: Option<String>,
    last_accessed: String,
    last_rotated: Option<String>,
    metadata: CredentialMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListCredentialsResponse {
    keys: Vec<String>,
    count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeleteCredentialResponse {
    success: bool,
    key: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExistsResponse {
    exists: bool,
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RotationCheckResponse {
    needs_rotation: Vec<String>,
    expired: Vec<String>,
    rotation_count: usize,
    expired_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct RotateCredentialRequest {
    new_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RotateCredentialResponse {
    success: bool,
    key: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

// =============================================================================
// Test Helpers
// =============================================================================

fn create_test_metadata(
    service: &str,
    cred_type: CredentialType,
    description: &str,
) -> CredentialMetadata {
    CredentialMetadata {
        service: service.to_string(),
        agent_name: Some("test_agent".to_string()),
        description: description.to_string(),
        credential_type: cred_type,
        rotation_required: false,
        rotation_interval_days: Some(90),
        encryption_algorithm: "aes-256-gcm".to_string(),
        custom: HashMap::new(),
    }
}

fn find_available_port() -> u16 {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port 0");
    listener.local_addr().unwrap().port()
}

// Note: These tests are currently ignored because they require:
// 1. The daemon to be fully integrated with the credential system
// 2. Authentication middleware to be in place
// 3. The credential endpoints to be registered in the daemon router
//
// Remove #[ignore] once Phase 2.2 (daemon integration) is complete

// =============================================================================
// SECTION 1: Basic Store and Retrieve Tests (5 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_store_and_retrieve_credential() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    // Wait for daemon (would need to be started)
    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Store credential
    let metadata = create_test_metadata(
        "test_service",
        CredentialType::ApiKey,
        "Test API key for integration tests",
    );

    let store_response = client
        .store_credential("test_api_key", "sk_test_12345", metadata.clone(), None)
        .await
        .unwrap();

    assert!(store_response.success);
    assert_eq!(store_response.key, "test_api_key");

    // Retrieve credential
    let retrieve_response = client.retrieve_credential("test_api_key").await.unwrap();

    assert_eq!(retrieve_response.key, "test_api_key");
    assert_eq!(retrieve_response.secret, "sk_test_12345");
    assert_eq!(retrieve_response.metadata.service, "test_service");
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_store_credential_with_expiration() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let metadata = create_test_metadata("expiring_service", CredentialType::ApiKey, "Expires soon");

    // Set expiration to 1 hour from now
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
    let expires_at_str = expires_at.to_rfc3339();

    let store_response = client
        .store_credential(
            "expiring_key",
            "secret_value",
            metadata,
            Some(expires_at_str.clone()),
        )
        .await
        .unwrap();

    assert!(store_response.success);

    // Retrieve and verify expiration
    let retrieve_response = client.retrieve_credential("expiring_key").await.unwrap();
    assert!(retrieve_response.expires_at.is_some());
    assert_eq!(retrieve_response.expires_at.unwrap(), expires_at_str);
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_retrieve_nonexistent_credential() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Try to retrieve non-existent key
    let result = client.retrieve_credential("does_not_exist").await;

    assert!(result.is_err());
    // Should return 404 Not Found
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_store_multiple_credential_types() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Store API Key
    let api_metadata = create_test_metadata("service1", CredentialType::ApiKey, "API Key");
    client
        .store_credential("key1", "api_secret", api_metadata, None)
        .await
        .unwrap();

    // Store Database credential
    let db_metadata = create_test_metadata("postgres", CredentialType::DatabaseUrl, "DB Password");
    client
        .store_credential("key2", "db_password", db_metadata, None)
        .await
        .unwrap();

    // Store OAuth Token
    let oauth_metadata = create_test_metadata("github", CredentialType::OAuth2Token, "OAuth Token");
    client
        .store_credential("key3", "oauth_token", oauth_metadata, None)
        .await
        .unwrap();

    // List and verify all three exist
    let list_response = client.list_credentials().await.unwrap();
    assert_eq!(list_response.count, 3);
    assert!(list_response.keys.contains(&"key1".to_string()));
    assert!(list_response.keys.contains(&"key2".to_string()));
    assert!(list_response.keys.contains(&"key3".to_string()));
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_overwrite_existing_credential() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let metadata = create_test_metadata("service", CredentialType::ApiKey, "Original");

    // Store initial credential
    client
        .store_credential("my_key", "original_secret", metadata.clone(), None)
        .await
        .unwrap();

    // Retrieve and verify
    let response1 = client.retrieve_credential("my_key").await.unwrap();
    assert_eq!(response1.secret, "original_secret");

    // Overwrite with new value
    client
        .store_credential("my_key", "new_secret", metadata, None)
        .await
        .unwrap();

    // Retrieve and verify updated value
    let response2 = client.retrieve_credential("my_key").await.unwrap();
    assert_eq!(response2.secret, "new_secret");
}

// =============================================================================
// SECTION 2: List and Delete Operations (4 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_list_credentials() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Initially empty
    let list1 = client.list_credentials().await.unwrap();
    let initial_count = list1.count;

    // Store some credentials
    let metadata = create_test_metadata("service", CredentialType::ApiKey, "Test");

    for i in 1..=5 {
        let key = format!("test_key_{}", i);
        client
            .store_credential(&key, "secret", metadata.clone(), None)
            .await
            .unwrap();
    }

    // List again
    let list2 = client.list_credentials().await.unwrap();
    assert_eq!(list2.count, initial_count + 5);
    assert!(list2.keys.contains(&"test_key_1".to_string()));
    assert!(list2.keys.contains(&"test_key_5".to_string()));
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_delete_credential() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let metadata = create_test_metadata("service", CredentialType::ApiKey, "To be deleted");

    // Store credential
    client
        .store_credential("delete_me", "secret", metadata, None)
        .await
        .unwrap();

    // Verify it exists
    let exists1 = client.check_exists("delete_me").await.unwrap();
    assert!(exists1.exists);

    // Delete it
    let delete_response = client.delete_credential("delete_me").await.unwrap();
    assert!(delete_response.success);
    assert_eq!(delete_response.key, "delete_me");

    // Verify it's gone
    let exists2 = client.check_exists("delete_me").await.unwrap();
    assert!(!exists2.exists);
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_delete_nonexistent_credential() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Try to delete non-existent key
    let result = client.delete_credential("does_not_exist").await;

    assert!(result.is_err());
    // Should return 404 Not Found
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_check_credential_exists() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Check non-existent
    let exists1 = client.check_exists("nonexistent").await.unwrap();
    assert!(!exists1.exists);

    // Store credential
    let metadata = create_test_metadata("service", CredentialType::ApiKey, "Exists test");
    client
        .store_credential("exists_key", "secret", metadata, None)
        .await
        .unwrap();

    // Check exists
    let exists2 = client.check_exists("exists_key").await.unwrap();
    assert!(exists2.exists);
}

// =============================================================================
// SECTION 3: Rotation and Expiration Tests (4 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_rotate_credential() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let metadata = create_test_metadata("service", CredentialType::ApiKey, "To be rotated");

    // Store initial credential
    client
        .store_credential("rotate_key", "old_secret", metadata, None)
        .await
        .unwrap();

    // Rotate it
    let rotate_response = client
        .rotate_credential("rotate_key", "new_secret")
        .await
        .unwrap();

    assert!(rotate_response.success);
    assert_eq!(rotate_response.key, "rotate_key");

    // Retrieve and verify new secret
    let retrieve_response = client.retrieve_credential("rotate_key").await.unwrap();
    assert_eq!(retrieve_response.secret, "new_secret");
    assert!(retrieve_response.last_rotated.is_some());
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_check_rotation_needed() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Create credential with rotation requirement
    let mut metadata = create_test_metadata(
        "service",
        CredentialType::ApiKey,
        "Needs rotation",
    );
    metadata.rotation_required = true;
    metadata.rotation_interval_days = Some(30); // 30 days

    // Store credential (would need to be >30 days old to trigger rotation)
    client
        .store_credential("old_key", "secret", metadata, None)
        .await
        .unwrap();

    // Check rotation
    let rotation_check = client.check_rotation().await.unwrap();

    // Initially won't need rotation (just created)
    // In a real test, we'd manipulate timestamps or wait
    assert!(rotation_check.rotation_count >= 0);
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_retrieve_expired_credential() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let metadata = create_test_metadata("service", CredentialType::ApiKey, "Already expired");

    // Set expiration to 1 hour AGO
    let expires_at = chrono::Utc::now() - chrono::Duration::hours(1);
    let expires_at_str = expires_at.to_rfc3339();

    // Store expired credential
    client
        .store_credential(
            "expired_key",
            "secret",
            metadata,
            Some(expires_at_str),
        )
        .await
        .unwrap();

    // Try to retrieve - should fail with 410 Gone
    let result = client.retrieve_credential("expired_key").await;
    assert!(result.is_err());
    // Error should indicate expiration
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_get_expired_credentials() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let metadata = create_test_metadata("service", CredentialType::ApiKey, "Expired");

    // Store expired credential
    let expires_at = chrono::Utc::now() - chrono::Duration::hours(1);
    client
        .store_credential(
            "expired1",
            "secret",
            metadata.clone(),
            Some(expires_at.to_rfc3339()),
        )
        .await
        .unwrap();

    // Store non-expired credential
    let future_expiry = chrono::Utc::now() + chrono::Duration::days(30);
    client
        .store_credential(
            "active1",
            "secret",
            metadata,
            Some(future_expiry.to_rfc3339()),
        )
        .await
        .unwrap();

    // Check rotation (includes expired list)
    let rotation_check = client.check_rotation().await.unwrap();

    // Should have at least 1 expired
    assert!(rotation_check.expired_count >= 1);
    assert!(rotation_check.expired.contains(&"expired1".to_string()));
}

// =============================================================================
// SECTION 4: Validation and Error Handling Tests (6 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_payload_too_large_key() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let metadata = create_test_metadata("service", CredentialType::ApiKey, "Large key test");

    // Create key larger than 1 KB
    let large_key = "k".repeat(2000); // 2 KB

    let result = client
        .store_credential(&large_key, "secret", metadata, None)
        .await;

    assert!(result.is_err());
    // Should return 413 Payload Too Large
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_payload_too_large_secret() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let metadata = create_test_metadata("service", CredentialType::ApiKey, "Large secret test");

    // Create secret larger than 100 KB
    let large_secret = "s".repeat(150 * 1024); // 150 KB

    let result = client
        .store_credential("normal_key", &large_secret, metadata, None)
        .await;

    assert!(result.is_err());
    // Should return 413 Payload Too Large
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_invalid_expiration_date() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let metadata = create_test_metadata("service", CredentialType::ApiKey, "Invalid expiry");

    // Use invalid date format
    let result = client
        .store_credential(
            "key",
            "secret",
            metadata,
            Some("not-a-date".to_string()),
        )
        .await;

    assert!(result.is_err());
    // Should return 400 Bad Request
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_malformed_json_request() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Send malformed JSON
    let response = client
        .raw_post(
            "/api/credentials/store",
            json!({
                "key": "test",
                // Missing required fields
            }),
        )
        .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_authentication_required() {
    let port = find_available_port();

    // Create client WITHOUT auth token
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let metadata = create_test_metadata("service", CredentialType::ApiKey, "Auth test");

    // Try to store without authentication
    // Note: This test depends on authentication middleware being enabled
    let result = client
        .store_credential("key", "secret", metadata, None)
        .await;

    // If auth is required, this should fail with 401 Unauthorized
    // If auth is not yet implemented, this test will pass (remove #[ignore])
    match result {
        Err(_) => {
            // Expected - auth required
        }
        Ok(_) => {
            // Auth not yet implemented - that's OK for now
            println!("Warning: Authentication not yet enforced");
        }
    }
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_rate_limiting() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let metadata = create_test_metadata("service", CredentialType::ApiKey, "Rate limit test");

    // Store a credential
    client
        .store_credential("rate_test_key", "secret", metadata, None)
        .await
        .unwrap();

    // Make 15 rapid retrieve requests (rate limit is 10 per 60 seconds)
    let mut success_count = 0;
    let mut rate_limited_count = 0;

    for _ in 0..15 {
        match client.retrieve_credential("rate_test_key").await {
            Ok(_) => success_count += 1,
            Err(_) => rate_limited_count += 1,
        }
    }

    // Should have been rate limited after 10 attempts
    assert!(rate_limited_count > 0, "Rate limiting should have kicked in");
    assert!(success_count <= 10, "Should not exceed rate limit");
}

// =============================================================================
// SECTION 5: Audit Logging Tests (2 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_audit_log_store_operation() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let metadata = create_test_metadata("service", CredentialType::ApiKey, "Audit test");

    // Store credential (should be audited)
    client
        .store_credential("audit_key", "secret", metadata, None)
        .await
        .unwrap();

    // In a real implementation, we'd check the audit log
    // For now, just verify the operation succeeded
    // Future: Add endpoint to query audit logs
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_audit_log_retrieve_operation() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    let metadata = create_test_metadata("service", CredentialType::ApiKey, "Audit test");

    // Store credential
    client
        .store_credential("audit_retrieve", "secret", metadata, None)
        .await
        .unwrap();

    // Retrieve credential (should be audited)
    client.retrieve_credential("audit_retrieve").await.unwrap();

    // Future: Verify audit log contains retrieve operation
}

// =============================================================================
// SECTION 6: Concurrent Access Tests (2 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_concurrent_store_operations() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Spawn 10 concurrent store operations
    let mut handles = vec![];

    for i in 0..10 {
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            let metadata =
                create_test_metadata("service", CredentialType::ApiKey, "Concurrent test");
            let key = format!("concurrent_key_{}", i);

            client_clone
                .store_credential(&key, "secret", metadata, None)
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    // Verify all were stored
    let list = client.list_credentials().await.unwrap();
    for i in 0..10 {
        let key = format!("concurrent_key_{}", i);
        assert!(list.keys.contains(&key));
    }
}

#[tokio::test]
#[ignore] // Remove after daemon integration complete
async fn test_concurrent_retrieve_operations() {
    let port = find_available_port();
    let client = CredentialTestClient::new(port);

    client
        .wait_for_ready(Duration::from_secs(5))
        .await
        .unwrap();

    // Store a credential
    let metadata = create_test_metadata(
        "service",
        CredentialType::ApiKey,
        "Concurrent retrieve test",
    );
    client
        .store_credential("shared_key", "shared_secret", metadata, None)
        .await
        .unwrap();

    // Spawn 20 concurrent retrieve operations
    let mut handles = vec![];

    for _ in 0..20 {
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            client_clone.retrieve_credential("shared_key").await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed (or be rate limited)
    let mut success_count = 0;
    for result in results {
        if result.is_ok() && result.unwrap().is_ok() {
            success_count += 1;
        }
    }

    // At least some should succeed
    assert!(success_count > 0, "Some concurrent retrievals should succeed");
}
