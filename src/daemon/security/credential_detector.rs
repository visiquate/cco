//! Credential detection module
//!
//! Detects common credential patterns in text to prevent accidental storage of secrets

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tracing::warn;

/// Credential match found in text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMatch {
    pub pattern_name: String,
    pub matched_text: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

/// Credential detector with pattern matching
pub struct CredentialDetector {
    patterns: Vec<(String, Regex)>,
}

impl CredentialDetector {
    /// Create a new credential detector with default patterns
    pub fn new() -> Self {
        let patterns = vec![
            // AWS Access Keys
            (
                "AWS Access Key ID".to_string(),
                Regex::new(r"(?i)(AKIA[0-9A-Z]{16})").unwrap(),
            ),
            (
                "AWS Secret Access Key".to_string(),
                Regex::new(
                    r#"(?i)(aws.{0,20}?(?:secret|access.?key).{0,20}?['"][0-9a-zA-Z/+=]{40}['"])"#,
                )
                .unwrap(),
            ),
            // API Keys
            (
                "Generic API Key".to_string(),
                Regex::new(r#"(?i)(api[_-]?key\s*[:=]\s*['"]?[0-9a-zA-Z\-_]{20,}['"]?)"#).unwrap(),
            ),
            (
                "API Key Assignment".to_string(),
                Regex::new(r#"(?i)(api.?key\s*=\s*['"][0-9a-zA-Z\-_]{16,}['"])"#).unwrap(),
            ),
            // Passwords
            (
                "Password Assignment".to_string(),
                Regex::new(r#"(?i)(password\s*[:=]\s*['"][^'"]{8,}['"])"#).unwrap(),
            ),
            (
                "Database URL with Password".to_string(),
                Regex::new(r"(?i)(postgres|mysql|mongodb)://[^:]+:[^@]+@").unwrap(),
            ),
            // JWT Tokens
            (
                "JWT Token".to_string(),
                Regex::new(r"(eyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+)").unwrap(),
            ),
            // Private Keys
            (
                "Private Key".to_string(),
                Regex::new(r"-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----").unwrap(),
            ),
            // OAuth/Bearer Tokens
            (
                "Bearer Token".to_string(),
                Regex::new(r"(?i)(bearer\s+[a-zA-Z0-9\-_\.]+)").unwrap(),
            ),
            // GitHub Tokens (minimum 20 chars after prefix)
            (
                "GitHub Token".to_string(),
                Regex::new(r"(ghp_[a-zA-Z0-9]{20,})").unwrap(),
            ),
            (
                "GitHub OAuth".to_string(),
                Regex::new(r"(gho_[a-zA-Z0-9]{20,})").unwrap(),
            ),
            // Slack Tokens
            (
                "Slack Token".to_string(),
                Regex::new(r"(xox[baprs]-[0-9a-zA-Z\-]+)").unwrap(),
            ),
            // Stripe Keys
            (
                "Stripe Key".to_string(),
                Regex::new(r"(sk_live_[0-9a-zA-Z]{24,})").unwrap(),
            ),
            // Google API Keys
            (
                "Google API Key".to_string(),
                Regex::new(r"(AIza[0-9A-Za-z\-_]{35})").unwrap(),
            ),
            // SSH Private Keys
            (
                "SSH Private Key".to_string(),
                Regex::new(r"-----BEGIN OPENSSH PRIVATE KEY-----").unwrap(),
            ),
            // Generic Secret Patterns
            (
                "Secret Assignment".to_string(),
                Regex::new(r#"(?i)(secret\s*[:=]\s*['"][^'"]{16,}['"])"#).unwrap(),
            ),
            (
                "Token Assignment".to_string(),
                Regex::new(r#"(?i)(token\s*[:=]\s*['"][a-zA-Z0-9\-_]{20,}['"])"#).unwrap(),
            ),
        ];

        Self { patterns }
    }

    /// Detect credentials in text
    pub fn detect(&self, text: &str) -> Vec<CredentialMatch> {
        let mut matches = Vec::new();

        for (pattern_name, regex) in &self.patterns {
            for capture in regex.captures_iter(text) {
                if let Some(matched) = capture.get(0) {
                    // Redact the actual credential value in logs
                    let matched_text = matched.as_str();
                    let redacted = Self::redact_credential(matched_text);

                    warn!(
                        "Detected credential pattern: {} (redacted: {})",
                        pattern_name, redacted
                    );

                    matches.push(CredentialMatch {
                        pattern_name: pattern_name.clone(),
                        matched_text: redacted,
                        start_pos: matched.start(),
                        end_pos: matched.end(),
                    });
                }
            }
        }

        matches
    }

    /// Redact credential value for logging
    pub fn redact_credential(text: &str) -> String {
        if text.len() <= 8 {
            return "*****".to_string();
        }

        let prefix_len = 4.min(text.len() / 4);
        let suffix_len = 4.min(text.len() / 4);

        let prefix = &text[..prefix_len];
        let suffix = &text[text.len() - suffix_len..];

        format!("{}***{}", prefix, suffix)
    }

    /// Check if text contains any credentials
    pub fn contains_credentials(&self, text: &str) -> bool {
        !self.detect(text).is_empty()
    }
}

impl Default for CredentialDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Global credential detector instance
static DETECTOR: OnceLock<CredentialDetector> = OnceLock::new();

/// Get global credential detector instance
pub fn get_detector() -> &'static CredentialDetector {
    DETECTOR.get_or_init(CredentialDetector::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_aws_access_key() {
        let detector = CredentialDetector::new();
        let text = "My AWS key is AKIAIOSFODNN7EXAMPLE";

        let matches = detector.detect(text);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].pattern_name, "AWS Access Key ID");
    }

    #[test]
    fn test_detect_api_key() {
        let detector = CredentialDetector::new();
        let text = "api_key=sk_test_1234567890abcdef";

        let matches = detector.detect(text);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_detect_password() {
        let detector = CredentialDetector::new();
        let text = "password = \"MySecurePassword123\"";

        let matches = detector.detect(text);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].pattern_name, "Password Assignment");
    }

    #[test]
    fn test_detect_jwt_token() {
        let detector = CredentialDetector::new();
        let text = "Token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";

        let matches = detector.detect(text);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].pattern_name, "JWT Token");
    }

    #[test]
    fn test_detect_private_key() {
        let detector = CredentialDetector::new();
        let text =
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----";

        let matches = detector.detect(text);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].pattern_name, "Private Key");
    }

    #[test]
    fn test_detect_database_url() {
        let detector = CredentialDetector::new();
        let text = "postgres://user:password123@localhost:5432/mydb";

        let matches = detector.detect(text);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].pattern_name, "Database URL with Password");
    }

    #[test]
    fn test_detect_github_token() {
        let detector = CredentialDetector::new();
        let text = "export GITHUB_TOKEN=ghp_1234567890abcdefghijklmnopqrstuv";

        let matches = detector.detect(text);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].pattern_name, "GitHub Token");
    }

    #[test]
    fn test_detect_bearer_token() {
        let detector = CredentialDetector::new();
        let text = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";

        let matches = detector.detect(text);
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_no_credentials() {
        let detector = CredentialDetector::new();
        let text = "This is a normal message with no credentials";

        let matches = detector.detect(text);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_contains_credentials() {
        let detector = CredentialDetector::new();
        assert!(detector.contains_credentials("api_key=secret123456789012345")); // 20+ chars
        assert!(!detector.contains_credentials("This is safe text"));
    }

    #[test]
    fn test_redact_credential() {
        let redacted = CredentialDetector::redact_credential("AKIAIOSFODNN7EXAMPLE");
        assert!(redacted.contains("***"));
        assert!(redacted.starts_with("AKIA"));

        let short = CredentialDetector::redact_credential("short");
        assert_eq!(short, "*****");
    }

    #[test]
    fn test_multiple_credentials() {
        let detector = CredentialDetector::new();
        let text = "api_key=\"abc123456789012345\" and password=\"secretpassword123\" and token=\"xyz789012345678901234567\"";

        let matches = detector.detect(text);
        assert!(matches.len() >= 2);
    }

    #[test]
    fn test_slack_token() {
        let detector = CredentialDetector::new();
        let text = "xoxb-1234567890-1234567890-abcdefghijklmnop";

        let matches = detector.detect(text);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].pattern_name, "Slack Token");
    }

    #[test]
    fn test_stripe_key() {
        let detector = CredentialDetector::new();
        let text = "sk_live_1234567890abcdefghijklmn"; // 24+ chars

        let matches = detector.detect(text);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].pattern_name, "Stripe Key");
    }

    #[test]
    fn test_google_api_key() {
        let detector = CredentialDetector::new();
        let text = "AIzaSyDaGmWKa4JsXZ-HjGw7ISLn_3namBGewQe";

        let matches = detector.detect(text);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].pattern_name, "Google API Key");
    }
}
