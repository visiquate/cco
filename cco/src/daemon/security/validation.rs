//! Input validation module
//!
//! Implements input validation including:
//! - Text field size limits (10 MB max)
//! - Metadata schema validation
//! - Request size limiting

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Maximum text field size (10 MB)
pub const MAX_TEXT_SIZE: usize = 10 * 1024 * 1024;

/// Maximum metadata size (100 KB)
pub const MAX_METADATA_SIZE: usize = 100 * 1024;

/// Maximum request body size (15 MB)
pub const MAX_REQUEST_SIZE: usize = 15 * 1024 * 1024;

/// Validation error types
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Text field too large: {size} bytes (max: {max} bytes)")]
    TextTooLarge { size: usize, max: usize },

    #[error("Metadata too large: {size} bytes (max: {max} bytes)")]
    MetadataTooLarge { size: usize, max: usize },

    #[error("Invalid metadata field: {field}")]
    InvalidMetadataField { field: String },

    #[error("Invalid metadata field type: {field} (expected: {expected})")]
    InvalidFieldType { field: String, expected: String },

    #[error("Request body too large: {size} bytes (max: {max} bytes)")]
    RequestTooLarge { size: usize, max: usize },
}

/// Validated metadata with typed fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedMetadata {
    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub confidence: Option<f64>,

    #[serde(default)]
    pub custom: HashMap<String, String>,
}

impl ValidatedMetadata {
    /// Create from arbitrary JSON metadata
    pub fn from_json(value: serde_json::Value) -> Result<Self, ValidationError> {
        // Validate metadata size
        let metadata_json = serde_json::to_string(&value)
            .map_err(|e| ValidationError::InvalidMetadataField {
                field: format!("JSON serialization failed: {}", e),
            })?;

        if metadata_json.len() > MAX_METADATA_SIZE {
            return Err(ValidationError::MetadataTooLarge {
                size: metadata_json.len(),
                max: MAX_METADATA_SIZE,
            });
        }

        // Extract typed fields
        let mut tags = Vec::new();
        let mut confidence = None;
        let mut custom = HashMap::new();

        if let serde_json::Value::Object(map) = value {
            for (key, val) in map {
                match key.as_str() {
                    "tags" => {
                        if let serde_json::Value::Array(arr) = val {
                            for item in arr {
                                if let serde_json::Value::String(s) = item {
                                    tags.push(s);
                                } else {
                                    return Err(ValidationError::InvalidFieldType {
                                        field: "tags".to_string(),
                                        expected: "array of strings".to_string(),
                                    });
                                }
                            }
                        } else {
                            return Err(ValidationError::InvalidFieldType {
                                field: "tags".to_string(),
                                expected: "array".to_string(),
                            });
                        }
                    }
                    "confidence" => {
                        if let serde_json::Value::Number(n) = val {
                            confidence = n.as_f64();
                        } else {
                            return Err(ValidationError::InvalidFieldType {
                                field: "confidence".to_string(),
                                expected: "number".to_string(),
                            });
                        }
                    }
                    _ => {
                        // Store in custom field as string
                        if let serde_json::Value::String(s) = val {
                            custom.insert(key, s);
                        } else {
                            custom.insert(key, val.to_string());
                        }
                    }
                }
            }
        }

        Ok(Self {
            tags,
            confidence,
            custom,
        })
    }

    /// Convert to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "tags": self.tags,
            "confidence": self.confidence,
            "custom": self.custom,
        })
    }

    /// Get as HashMap for backward compatibility
    pub fn to_hashmap(&self) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();

        if !self.tags.is_empty() {
            map.insert(
                "tags".to_string(),
                serde_json::Value::Array(
                    self.tags.iter().map(|s| serde_json::Value::String(s.clone())).collect()
                ),
            );
        }

        if let Some(conf) = self.confidence {
            map.insert(
                "confidence".to_string(),
                serde_json::Value::Number(serde_json::Number::from_f64(conf).unwrap()),
            );
        }

        for (key, value) in &self.custom {
            map.insert(key.clone(), serde_json::Value::String(value.clone()));
        }

        map
    }
}

impl Default for ValidatedMetadata {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            confidence: None,
            custom: HashMap::new(),
        }
    }
}

/// Validate text field size
pub fn validate_text_size(text: &str) -> Result<(), ValidationError> {
    let size = text.len();
    if size > MAX_TEXT_SIZE {
        return Err(ValidationError::TextTooLarge {
            size,
            max: MAX_TEXT_SIZE,
        });
    }
    Ok(())
}

/// Validate request body size
pub fn validate_request_size(size: usize) -> Result<(), ValidationError> {
    if size > MAX_REQUEST_SIZE {
        return Err(ValidationError::RequestTooLarge {
            size,
            max: MAX_REQUEST_SIZE,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_text_size_ok() {
        let text = "Hello, world!";
        assert!(validate_text_size(text).is_ok());
    }

    #[test]
    fn test_validate_text_size_too_large() {
        let text = "x".repeat(MAX_TEXT_SIZE + 1);
        let result = validate_text_size(&text);
        assert!(result.is_err());

        if let Err(ValidationError::TextTooLarge { size, max }) = result {
            assert_eq!(size, MAX_TEXT_SIZE + 1);
            assert_eq!(max, MAX_TEXT_SIZE);
        } else {
            panic!("Expected TextTooLarge error");
        }
    }

    #[test]
    fn test_validate_request_size_ok() {
        assert!(validate_request_size(1024).is_ok());
    }

    #[test]
    fn test_validate_request_size_too_large() {
        let result = validate_request_size(MAX_REQUEST_SIZE + 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_validated_metadata_from_json() {
        let json = serde_json::json!({
            "tags": ["test", "example"],
            "confidence": 0.95,
            "source": "manual"
        });

        let metadata = ValidatedMetadata::from_json(json).unwrap();
        assert_eq!(metadata.tags, vec!["test", "example"]);
        assert_eq!(metadata.confidence, Some(0.95));
        assert_eq!(metadata.custom.get("source"), Some(&"manual".to_string()));
    }

    #[test]
    fn test_validated_metadata_empty() {
        let json = serde_json::json!({});
        let metadata = ValidatedMetadata::from_json(json).unwrap();
        assert!(metadata.tags.is_empty());
        assert!(metadata.confidence.is_none());
        assert!(metadata.custom.is_empty());
    }

    #[test]
    fn test_validated_metadata_invalid_tags_type() {
        let json = serde_json::json!({
            "tags": "not-an-array"
        });

        let result = ValidatedMetadata::from_json(json);
        assert!(result.is_err());

        if let Err(ValidationError::InvalidFieldType { field, .. }) = result {
            assert_eq!(field, "tags");
        } else {
            panic!("Expected InvalidFieldType error");
        }
    }

    #[test]
    fn test_validated_metadata_invalid_confidence_type() {
        let json = serde_json::json!({
            "confidence": "not-a-number"
        });

        let result = ValidatedMetadata::from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validated_metadata_too_large() {
        let large_value = "x".repeat(MAX_METADATA_SIZE);
        let json = serde_json::json!({
            "custom_field": large_value
        });

        let result = ValidatedMetadata::from_json(json);
        assert!(result.is_err());

        if let Err(ValidationError::MetadataTooLarge { size, max }) = result {
            assert!(size > MAX_METADATA_SIZE);
            assert_eq!(max, MAX_METADATA_SIZE);
        } else {
            panic!("Expected MetadataTooLarge error");
        }
    }

    #[test]
    fn test_validated_metadata_to_json() {
        let metadata = ValidatedMetadata {
            tags: vec!["test".to_string()],
            confidence: Some(0.9),
            custom: {
                let mut map = HashMap::new();
                map.insert("key".to_string(), "value".to_string());
                map
            },
        };

        let json = metadata.to_json();
        assert_eq!(json["tags"], serde_json::json!(["test"]));
        assert_eq!(json["confidence"], serde_json::json!(0.9));
        assert_eq!(json["custom"]["key"], serde_json::json!("value"));
    }

    #[test]
    fn test_validated_metadata_to_hashmap() {
        let metadata = ValidatedMetadata {
            tags: vec!["test".to_string()],
            confidence: Some(0.9),
            custom: HashMap::new(),
        };

        let map = metadata.to_hashmap();
        assert!(map.contains_key("tags"));
        assert!(map.contains_key("confidence"));
    }
}
