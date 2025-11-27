//! OIDC configuration for Authentik integration

use serde::{Deserialize, Serialize};

/// OIDC configuration for device flow authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    /// OIDC client ID (e.g., "cco-cli")
    pub client_id: String,

    /// OIDC issuer URL (e.g., "https://auth.visiquate.com/application/o/cco-cli/")
    pub issuer: String,

    /// Device authorization endpoint
    pub device_auth_endpoint: String,

    /// Token endpoint
    pub token_endpoint: String,

    /// User info endpoint (optional)
    pub userinfo_endpoint: Option<String>,

    /// Additional scopes to request (beyond openid)
    #[serde(default)]
    pub scopes: Vec<String>,
}

impl OidcConfig {
    /// Create a new OIDC configuration
    pub fn new(
        client_id: String,
        issuer: String,
        device_auth_endpoint: String,
        token_endpoint: String,
    ) -> Self {
        Self {
            client_id,
            issuer,
            device_auth_endpoint,
            token_endpoint,
            userinfo_endpoint: None,
            scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.client_id.is_empty() {
            return Err("client_id cannot be empty".to_string());
        }

        if self.issuer.is_empty() {
            return Err("issuer cannot be empty".to_string());
        }

        if self.device_auth_endpoint.is_empty() {
            return Err("device_auth_endpoint cannot be empty".to_string());
        }

        if self.token_endpoint.is_empty() {
            return Err("token_endpoint cannot be empty".to_string());
        }

        // Validate URLs
        if !self.issuer.starts_with("https://") && !self.issuer.starts_with("http://") {
            return Err("issuer must be a valid HTTP(S) URL".to_string());
        }

        if !self.device_auth_endpoint.starts_with("https://")
            && !self.device_auth_endpoint.starts_with("http://")
        {
            return Err("device_auth_endpoint must be a valid HTTP(S) URL".to_string());
        }

        if !self.token_endpoint.starts_with("https://")
            && !self.token_endpoint.starts_with("http://")
        {
            return Err("token_endpoint must be a valid HTTP(S) URL".to_string());
        }

        Ok(())
    }

    /// Get scope string for requests
    pub fn scope_string(&self) -> String {
        self.scopes.join(" ")
    }
}

impl Default for OidcConfig {
    /// Default configuration for VisiQuate Authentik
    fn default() -> Self {
        Self {
            client_id: "cco-cli".to_string(),
            issuer: "https://auth.visiquate.com/application/o/cco-cli/".to_string(),
            device_auth_endpoint: "https://auth.visiquate.com/application/o/device/".to_string(),
            token_endpoint: "https://auth.visiquate.com/application/o/token/".to_string(),
            userinfo_endpoint: Some(
                "https://auth.visiquate.com/application/o/userinfo/".to_string(),
            ),
            scopes: vec![
                "openid".to_string(),
                "profile".to_string(),
                "email".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OidcConfig::default();
        assert_eq!(config.client_id, "cco-cli");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = OidcConfig::default();

        // Valid config
        assert!(config.validate().is_ok());

        // Invalid: empty client_id
        config.client_id = String::new();
        assert!(config.validate().is_err());

        // Reset
        config = OidcConfig::default();

        // Invalid: bad URL
        config.issuer = "not-a-url".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_scope_string() {
        let config = OidcConfig::default();
        let scope_str = config.scope_string();
        assert!(scope_str.contains("openid"));
        assert!(scope_str.contains("profile"));
        assert!(scope_str.contains("email"));
    }

    #[test]
    fn test_custom_config() {
        let config = OidcConfig::new(
            "custom-client".to_string(),
            "https://auth.example.com/".to_string(),
            "https://auth.example.com/device".to_string(),
            "https://auth.example.com/token".to_string(),
        );

        assert_eq!(config.client_id, "custom-client");
        assert!(config.validate().is_ok());
    }
}
