//! Authentication module for CCO
//!
//! Provides OIDC device flow authentication with cco-api.visiquate.com

pub mod config;
mod device_flow;
mod token_storage;

pub use config::OidcConfig;
pub use device_flow::{DeviceFlowClient, DeviceFlowError, DeviceFlowResponse, TokenResponse};
pub use token_storage::{TokenInfo, TokenStorage};

use anyhow::Result;

const AUTH_API_URL: &str = "https://cco-api.visiquate.com";

/// Perform login via OIDC device flow
pub async fn login() -> Result<()> {
    let client = DeviceFlowClient::new(AUTH_API_URL);
    let storage = TokenStorage::new()?;

    println!("🔐 Authenticating with VisiQuate...\n");

    // Start device flow
    let flow = client.start_device_flow().await?;

    // Display user instructions with better formatting
    println!("Visit: {}", flow.verification_uri);
    println!("Code: {}", flow.user_code);
    println!();
    println!("Waiting for authentication... ⏳");

    // Poll for completion
    let tokens = client.poll_for_tokens(&flow).await?;

    // Store tokens
    storage.store_tokens(&tokens)?;

    println!();
    println!("✅ Successfully logged in!");
    println!("   Storage: {}", storage.get_backend());

    Ok(())
}

/// Perform logout (clear stored tokens)
pub async fn logout() -> Result<()> {
    let storage = TokenStorage::new()?;

    if !storage.has_tokens()? {
        println!("ℹ️  Not currently logged in");
        return Ok(());
    }

    storage.clear_tokens()?;

    println!("✅ Logout successful!");
    println!("   Tokens cleared");

    Ok(())
}

/// Check if user is authenticated
pub fn is_authenticated() -> Result<bool> {
    let storage = TokenStorage::new()?;
    storage.has_valid_tokens()
}

/// Get access token (refresh if needed)
pub async fn get_access_token() -> Result<String> {
    let storage = TokenStorage::new()?;
    let client = DeviceFlowClient::new(AUTH_API_URL);

    let tokens = storage.get_tokens()?;

    // Check if token is expired or about to expire (within 5 minutes)
    if tokens.is_expired(300) {
        // Refresh token
        let new_tokens = client.refresh_token(&tokens.refresh_token).await?;
        storage.store_tokens(&new_tokens)?;
        Ok(new_tokens.access_token)
    } else {
        Ok(tokens.access_token)
    }
}
