//! Data models for credentials and metadata

#[cfg(test)]
use chrono::Duration;
use chrono::{DateTime, Utc};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A credential with metadata
#[derive(Clone)]
pub struct Credential {
    /// Unique identifier (e.g., "salesforce_api_token")
    pub key: String,

    /// The actual secret value (zeroized on drop)
    pub secret: SecretString,

    /// When credential was created
    pub created_at: DateTime<Utc>,

    /// Optional expiration time
    pub expires_at: Option<DateTime<Utc>>,

    /// Last accessed timestamp
    pub last_accessed: DateTime<Utc>,

    /// Last rotation timestamp
    pub last_rotated: Option<DateTime<Utc>>,

    /// Credential metadata
    pub metadata: CredentialMetadata,
}

// Manual serialization to handle SecretString properly
impl Serialize for Credential {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Credential", 6)?;
        state.serialize_field("key", &self.key)?;
        // Skip secret - never serialize it
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("expires_at", &self.expires_at)?;
        state.serialize_field("last_accessed", &self.last_accessed)?;
        state.serialize_field("last_rotated", &self.last_rotated)?;
        state.serialize_field("metadata", &self.metadata)?;
        state.end()
    }
}

// Manual deserialization to handle SecretString properly
impl<'de> Deserialize<'de> for Credential {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct CredentialVisitor;

        impl<'de> Visitor<'de> for CredentialVisitor {
            type Value = Credential;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Credential")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut key = None;
                let mut created_at = None;
                let mut expires_at = None;
                let mut last_accessed = None;
                let mut last_rotated = None;
                let mut metadata = None;

                while let Some(k) = map.next_key()? {
                    match k {
                        "key" => key = Some(map.next_value()?),
                        "created_at" => created_at = Some(map.next_value()?),
                        "expires_at" => expires_at = Some(map.next_value()?),
                        "last_accessed" => last_accessed = Some(map.next_value()?),
                        "last_rotated" => last_rotated = Some(map.next_value()?),
                        "metadata" => metadata = Some(map.next_value()?),
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(Credential {
                    key: key.ok_or_else(|| de::Error::missing_field("key"))?,
                    secret: SecretString::new(String::new()), // Cannot deserialize secrets
                    created_at: created_at.ok_or_else(|| de::Error::missing_field("created_at"))?,
                    expires_at,
                    last_accessed: last_accessed
                        .ok_or_else(|| de::Error::missing_field("last_accessed"))?,
                    last_rotated,
                    metadata: metadata.ok_or_else(|| de::Error::missing_field("metadata"))?,
                })
            }
        }

        deserializer.deserialize_struct(
            "Credential",
            &[
                "key",
                "created_at",
                "expires_at",
                "last_accessed",
                "last_rotated",
                "metadata",
            ],
            CredentialVisitor,
        )
    }
}

impl fmt::Debug for Credential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credential")
            .field("key", &self.key)
            .field("secret", &"***REDACTED***")
            .field("created_at", &self.created_at)
            .field("expires_at", &self.expires_at)
            .field("last_accessed", &self.last_accessed)
            .field("last_rotated", &self.last_rotated)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl Credential {
    /// Create a new credential with default timestamps
    pub fn new(key: String, secret: String, metadata: CredentialMetadata) -> Self {
        Self {
            key,
            secret: SecretString::new(secret),
            created_at: Utc::now(),
            expires_at: None,
            last_accessed: Utc::now(),
            last_rotated: None,
            metadata,
        }
    }

    /// Check if credential has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now()
        } else {
            false
        }
    }

    /// Check if credential needs rotation
    pub fn needs_rotation(&self) -> bool {
        if !self.metadata.rotation_required {
            return false;
        }

        if let Some(interval_days) = self.metadata.rotation_interval_days {
            if let Some(last_rotated) = self.last_rotated {
                let elapsed = Utc::now().signed_duration_since(last_rotated);
                elapsed.num_days() >= interval_days as i64
            } else {
                // Never rotated
                true
            }
        } else {
            false
        }
    }
}

/// Metadata about a credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMetadata {
    /// Service this credential is for (e.g., "salesforce", "github")
    pub service: String,

    /// Which agent uses this credential
    pub agent_name: Option<String>,

    /// Human-readable description
    pub description: String,

    /// Type of credential
    pub credential_type: CredentialType,

    /// Whether rotation is required
    pub rotation_required: bool,

    /// Rotation interval in days (if rotation required)
    pub rotation_interval_days: Option<u32>,

    /// Which encryption algorithm was used
    pub encryption_algorithm: String,

    /// Additional custom key-value pairs
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

impl Default for CredentialMetadata {
    fn default() -> Self {
        Self {
            service: String::new(),
            agent_name: None,
            description: String::new(),
            credential_type: CredentialType::Generic,
            rotation_required: false,
            rotation_interval_days: None,
            encryption_algorithm: "aes-256-gcm".to_string(),
            custom: HashMap::new(),
        }
    }
}

/// Types of credentials
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CredentialType {
    /// API key or token
    ApiKey,

    /// Password
    Password,

    /// Generic token
    Token,

    /// TLS/SSL certificate
    Certificate,

    /// Private key (RSA, EC, etc.)
    PrivateKey,

    /// OAuth 2.0 access token
    OAuth2Token,

    /// Database connection URL
    DatabaseUrl,

    /// Generic/unspecified
    Generic,
}

impl fmt::Display for CredentialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ApiKey => write!(f, "api_key"),
            Self::Password => write!(f, "password"),
            Self::Token => write!(f, "token"),
            Self::Certificate => write!(f, "certificate"),
            Self::PrivateKey => write!(f, "private_key"),
            Self::OAuth2Token => write!(f, "oauth2_token"),
            Self::DatabaseUrl => write!(f, "database_url"),
            Self::Generic => write!(f, "generic"),
        }
    }
}

impl std::str::FromStr for CredentialType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "api_key" | "apikey" => Ok(Self::ApiKey),
            "password" => Ok(Self::Password),
            "token" => Ok(Self::Token),
            "certificate" | "cert" => Ok(Self::Certificate),
            "private_key" | "privatekey" => Ok(Self::PrivateKey),
            "oauth2_token" | "oauth2" => Ok(Self::OAuth2Token),
            "database_url" | "db_url" => Ok(Self::DatabaseUrl),
            "generic" => Ok(Self::Generic),
            _ => Err(format!("Unknown credential type: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_creation() {
        let metadata = CredentialMetadata {
            service: "test".to_string(),
            credential_type: CredentialType::ApiKey,
            ..Default::default()
        };

        let cred = Credential::new("test_key".to_string(), "test_secret".to_string(), metadata);

        assert_eq!(cred.key, "test_key");
        assert_eq!(cred.metadata.service, "test");
    }

    #[test]
    fn test_credential_expiration() {
        let mut cred = Credential::new(
            "test".to_string(),
            "secret".to_string(),
            CredentialMetadata::default(),
        );

        // Not expired by default
        assert!(!cred.is_expired());

        // Set expiration to past
        cred.expires_at = Some(Utc::now() - Duration::days(1));
        assert!(cred.is_expired());

        // Set expiration to future
        cred.expires_at = Some(Utc::now() + Duration::days(1));
        assert!(!cred.is_expired());
    }

    #[test]
    fn test_credential_rotation_needed() {
        let mut metadata = CredentialMetadata::default();
        metadata.rotation_required = true;
        metadata.rotation_interval_days = Some(90);

        let mut cred = Credential::new("test".to_string(), "secret".to_string(), metadata);

        // Never rotated
        assert!(cred.needs_rotation());

        // Recently rotated
        cred.last_rotated = Some(Utc::now() - Duration::days(30));
        assert!(!cred.needs_rotation());

        // Rotation overdue
        cred.last_rotated = Some(Utc::now() - Duration::days(100));
        assert!(cred.needs_rotation());
    }

    #[test]
    fn test_credential_type_parsing() {
        use std::str::FromStr;

        assert_eq!(
            CredentialType::from_str("api_key").unwrap(),
            CredentialType::ApiKey
        );
        assert_eq!(
            CredentialType::from_str("password").unwrap(),
            CredentialType::Password
        );
        assert_eq!(
            CredentialType::from_str("oauth2").unwrap(),
            CredentialType::OAuth2Token
        );
        assert!(CredentialType::from_str("invalid").is_err());
    }

    #[test]
    fn test_credential_debug_no_leak() {
        let cred = Credential::new(
            "test_key".to_string(),
            "super_secret_password".to_string(),
            CredentialMetadata::default(),
        );

        let debug_str = format!("{:?}", cred);
        assert!(debug_str.contains("REDACTED"));
        assert!(!debug_str.contains("super_secret_password"));
    }
}
