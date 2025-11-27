//! OIDC Device Flow implementation

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Device flow client
pub struct DeviceFlowClient {
    api_url: String,
    client: reqwest::Client,
}

/// Device flow initialization response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFlowResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

/// Device flow errors
#[derive(Debug, thiserror::Error)]
pub enum DeviceFlowError {
    #[error("Authorization pending - waiting for user to complete authentication")]
    AuthorizationPending,

    #[error("Slow down - polling too fast")]
    SlowDown,

    #[error("Access denied - user rejected the request")]
    AccessDenied,

    #[error("Expired token - device code expired")]
    ExpiredToken,

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Other error: {0}")]
    Other(String),
}

impl DeviceFlowClient {
    /// Create a new device flow client
    pub fn new(api_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("cco/client")
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            api_url: api_url.to_string(),
            client,
        }
    }

    /// Start device flow
    pub async fn start_device_flow(&self) -> Result<DeviceFlowResponse> {
        let url = format!("{}/auth/device/code", self.api_url);

        let response = self
            .client
            .post(&url)
            .send()
            .await
            .context("Failed to initiate device flow")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Device flow initiation failed (HTTP {}): {}",
                status,
                body
            ));
        }

        let flow: DeviceFlowResponse = response
            .json()
            .await
            .context("Failed to parse device flow response")?;

        Ok(flow)
    }

    /// Poll for tokens
    pub async fn poll_for_tokens(&self, flow: &DeviceFlowResponse) -> Result<TokenResponse> {
        let url = format!("{}/auth/device/token", self.api_url);
        let interval = Duration::from_secs(flow.interval);
        let max_attempts = (flow.expires_in / flow.interval) + 5;

        for attempt in 0..max_attempts {
            if attempt > 0 {
                tokio::time::sleep(interval).await;
            }

            let response = self
                .client
                .post(&url)
                .json(&serde_json::json!({
                    "device_code": flow.device_code,
                }))
                .send()
                .await
                .context("Failed to poll for tokens")?;

            let status = response.status();

            if status.is_success() {
                let tokens: TokenResponse = response
                    .json()
                    .await
                    .context("Failed to parse token response")?;
                return Ok(tokens);
            }

            // Parse error response
            #[derive(Deserialize)]
            struct ErrorResponse {
                error: String,
                #[allow(dead_code)]
                error_description: Option<String>,
            }

            let error_body = response
                .json::<ErrorResponse>()
                .await
                .context("Failed to parse error response")?;

            match error_body.error.as_str() {
                "authorization_pending" => {
                    // Continue polling
                    continue;
                }
                "slow_down" => {
                    // Wait longer before next poll
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
                "access_denied" => {
                    return Err(DeviceFlowError::AccessDenied.into());
                }
                "expired_token" => {
                    return Err(DeviceFlowError::ExpiredToken.into());
                }
                _ => {
                    return Err(DeviceFlowError::Other(error_body.error).into());
                }
            }
        }

        Err(anyhow!("Device flow timed out"))
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let url = format!("{}/auth/token/refresh", self.api_url);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "refresh_token": refresh_token,
            }))
            .send()
            .await
            .context("Failed to refresh token")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if status == reqwest::StatusCode::UNAUTHORIZED {
                return Err(anyhow!(
                    "Refresh token expired or invalid. Please run 'cco login' again."
                ));
            }

            return Err(anyhow!(
                "Token refresh failed (HTTP {}): {}",
                status,
                body
            ));
        }

        let tokens: TokenResponse = response
            .json()
            .await
            .context("Failed to parse refresh response")?;

        Ok(tokens)
    }
}
