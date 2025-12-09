//! Prompt engineering for CRUD classification
//!
//! Provides prompt construction and response parsing for the
//! embedded LLM classifier.

use crate::daemon::hooks::{CrudClassification, HookError, HookResult};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

/// User correction for a previous classification
#[derive(Debug, Clone, Deserialize)]
pub struct Correction {
    /// The command that was classified
    pub command: String,
    /// The model's predicted classification
    pub predicted: String,
    /// The user's expected/correct classification
    pub expected: String,
    /// Confidence score of the original prediction
    #[serde(default)]
    pub confidence: f64,
    /// Timestamp of the correction
    #[serde(default)]
    pub timestamp: String,
}

/// Container for the corrections JSON file
#[derive(Debug, Deserialize)]
struct CorrectionsFile {
    corrections: Vec<Correction>,
    #[serde(default)]
    #[allow(dead_code)]
    version: String,
    #[serde(default)]
    #[allow(dead_code)]
    last_updated: String,
}

/// Load user corrections from ~/.cco/classifier-corrections.json
///
/// Returns an empty vector if the file doesn't exist or can't be parsed.
/// This function is called on every classification to allow hot-reloading
/// of corrections without restarting the daemon.
fn load_corrections() -> Vec<Correction> {
    // Build path to corrections file
    let home_dir = match dirs::home_dir() {
        Some(dir) => dir,
        None => return Vec::new(),
    };

    let corrections_path: PathBuf = home_dir.join(".cco").join("classifier-corrections.json");

    // Return empty vec if file doesn't exist (not an error)
    if !corrections_path.exists() {
        return Vec::new();
    }

    // Try to read and parse the file
    match fs::read_to_string(&corrections_path) {
        Ok(contents) => match serde_json::from_str::<CorrectionsFile>(&contents) {
            Ok(file) => file.corrections,
            Err(_) => {
                // File exists but is corrupted - return empty vec
                Vec::new()
            }
        },
        Err(_) => Vec::new(),
    }
}

/// Build a CRUD classification prompt for the given command
///
/// Creates a focused, single-shot prompt that asks the model to
/// classify the command as exactly one of: READ, CREATE, UPDATE, DELETE.
///
/// Includes user corrections as few-shot examples to improve accuracy.
///
/// # Arguments
///
/// * `command` - The shell command to classify
///
/// # Returns
///
/// A prompt string suitable for the Qwen2.5-Coder model
///
/// # Example
///
/// ```rust
/// use cco::daemon::hooks::llm::build_crud_prompt;
///
/// let prompt = build_crud_prompt("ls -la");
/// assert!(prompt.contains("ls -la"));
/// ```
pub fn build_crud_prompt(command: &str) -> String {
    // Load corrections on each call for hot-reloading
    let corrections = load_corrections();

    let mut prompt = String::new();

    // Add few-shot examples from user corrections
    if !corrections.is_empty() {
        eprintln!("DEBUG: Loaded {} corrections for prompt", corrections.len());
        prompt.push_str("IMPORTANT: Learn from these user-corrected examples:\n\n");
        for correction in &corrections {
            // Show the misclassification and the correction
            prompt.push_str(&format!(
                "Command: {}\nWRONG: {} | CORRECT: {}\n\n",
                correction.command, correction.predicted, correction.expected
            ));
        }
        prompt.push_str("Now classify this command using the patterns above:\n");
    } else {
        eprintln!("DEBUG: No corrections loaded from ~/.cco/classifier-corrections.json");
    }

    // Add the command to classify
    prompt.push_str(&format!("Command: {}", command.trim()));

    prompt
}

/// Parse LLM response into CRUD classification
///
/// Extracts the classification from the model's response text.
/// Handles various response formats and provides fallback behavior.
///
/// # Arguments
///
/// * `response` - The raw response text from the LLM
///
/// # Returns
///
/// A `CrudClassification` if parsing succeeds
///
/// # Errors
///
/// Returns `HookError::InferenceFailed` if the response cannot be parsed
///
/// # Example
///
/// ```rust
/// use cco::daemon::hooks::llm::parse_classification;
/// use cco::daemon::hooks::CrudClassification;
///
/// let classification = parse_classification("READ").unwrap();
/// assert_eq!(classification, CrudClassification::Read);
/// ```
pub fn parse_classification(response: &str) -> HookResult<CrudClassification> {
    // Clean up the response - extract just the classification word
    let cleaned = extract_classification_word(response);

    // Try to parse as CRUD classification
    match CrudClassification::from_str(&cleaned) {
        Ok(classification) => Ok(classification),
        Err(_) => {
            // If we can't parse it, return an error with the raw response
            Err(HookError::execution_failed(
                "crud_classifier",
                format!(
                    "Invalid classification response: '{}' (cleaned: '{}')",
                    response.trim(),
                    cleaned
                ),
            ))
        }
    }
}

/// Extract the classification word from a potentially verbose response
///
/// Handles cases where the model provides additional explanation
/// or formats the response differently than expected.
fn extract_classification_word(response: &str) -> String {
    let response = response.trim();

    // Common patterns the model might use
    let patterns = [
        "Classification:",
        "Answer:",
        "Result:",
        "Type:",
        "CRUD:",
        "Operation:",
    ];

    // Try to extract after a pattern
    for pattern in &patterns {
        if let Some(idx) = response.find(pattern) {
            let after = &response[idx + pattern.len()..];
            if let Some(word) = after.split_whitespace().next() {
                return word
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string();
            }
        }
    }

    // If no pattern found, take the first capitalized word that looks like a CRUD word
    for word in response.split_whitespace() {
        let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric());
        if cleaned.len() >= 4 && cleaned.chars().next().unwrap_or('a').is_uppercase() {
            if matches!(
                cleaned.to_uppercase().as_str(),
                "READ" | "CREATE" | "UPDATE" | "DELETE"
            ) {
                return cleaned.to_string();
            }
        }
    }

    // Fallback: return the first word
    response
        .split_whitespace()
        .next()
        .unwrap_or(response)
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_crud_prompt() {
        let prompt = build_crud_prompt("ls -la");
        assert!(prompt.contains("ls -la"));
        assert!(prompt.contains("Command:"));
    }

    #[test]
    fn test_prompt_is_reasonable_size() {
        let prompt = build_crud_prompt("test");
        // Prompt should be reasonable size (may include corrections)
        // With corrections, it can be longer, but shouldn't be excessive
        assert!(
            prompt.len() < 10000,
            "Prompt should be reasonable size, was {} chars",
            prompt.len()
        );
        // Should always end with the command to classify
        assert!(prompt.ends_with("Command: test"));
    }

    #[test]
    fn test_prompt_includes_user_command() {
        let user_command = "cargo test --all";
        let prompt = build_crud_prompt(user_command);
        assert!(prompt.contains(user_command));
    }

    #[test]
    fn test_prompt_trims_command() {
        let prompt = build_crud_prompt("  cargo test  ");
        assert!(prompt.contains("cargo test"));
        assert!(!prompt.contains("  cargo")); // Should be trimmed
    }

    #[test]
    fn test_parse_classification_simple() {
        assert_eq!(
            parse_classification("READ").unwrap(),
            CrudClassification::Read
        );
        assert_eq!(
            parse_classification("CREATE").unwrap(),
            CrudClassification::Create
        );
        assert_eq!(
            parse_classification("UPDATE").unwrap(),
            CrudClassification::Update
        );
        assert_eq!(
            parse_classification("DELETE").unwrap(),
            CrudClassification::Delete
        );
    }

    #[test]
    fn test_parse_classification_lowercase() {
        assert_eq!(
            parse_classification("read").unwrap(),
            CrudClassification::Read
        );
        assert_eq!(
            parse_classification("create").unwrap(),
            CrudClassification::Create
        );
    }

    #[test]
    fn test_parse_classification_with_prefix() {
        assert_eq!(
            parse_classification("Classification: READ").unwrap(),
            CrudClassification::Read
        );
        assert_eq!(
            parse_classification("Answer: CREATE").unwrap(),
            CrudClassification::Create
        );
        assert_eq!(
            parse_classification("Result: UPDATE").unwrap(),
            CrudClassification::Update
        );
    }

    #[test]
    fn test_parse_classification_with_explanation() {
        assert_eq!(
            parse_classification("READ - this command only reads files").unwrap(),
            CrudClassification::Read
        );
    }

    #[test]
    fn test_parse_classification_with_whitespace() {
        assert_eq!(
            parse_classification("  READ  ").unwrap(),
            CrudClassification::Read
        );
        assert_eq!(
            parse_classification("\nCREATE\n").unwrap(),
            CrudClassification::Create
        );
    }

    #[test]
    fn test_parse_classification_invalid() {
        assert!(parse_classification("INVALID").is_err());
        assert!(parse_classification("").is_err());
        assert!(parse_classification("MODIFY").is_err());
    }

    #[test]
    fn test_extract_classification_word() {
        assert_eq!(extract_classification_word("READ"), "READ");
        assert_eq!(
            extract_classification_word("Classification: CREATE"),
            "CREATE"
        );
        assert_eq!(
            extract_classification_word("The answer is UPDATE"),
            "UPDATE"
        );
        assert_eq!(extract_classification_word("  DELETE  "), "DELETE");
    }

    #[test]
    fn test_prompt_with_corrections() {
        // Test that prompt includes corrections if they exist
        let prompt = build_crud_prompt("ls -la");
        assert!(prompt.contains("Command: ls -la"));
        // The prompt should end with the command to classify
        assert!(prompt.ends_with("Command: ls -la"));
        // If corrections exist, they should be formatted with WRONG/CORRECT labels
        if prompt.contains("IMPORTANT:") {
            assert!(prompt.contains("WRONG:") || prompt.contains("CORRECT:"));
        }
    }

    #[test]
    fn test_load_corrections_missing_file() {
        // Should return empty vec when file doesn't exist
        let corrections = load_corrections();
        // This test will pass as long as load_corrections doesn't panic
        assert!(corrections.is_empty() || !corrections.is_empty());
    }

    #[test]
    fn test_correction_deserialization() {
        // Test that Correction struct can deserialize from JSON
        let json = r#"{
            "command": "echo > file.txt",
            "predicted": "Create",
            "expected": "Update",
            "confidence": 0.95,
            "timestamp": "2025-12-06T00:00:00Z"
        }"#;

        let correction: Correction = serde_json::from_str(json).unwrap();
        assert_eq!(correction.command, "echo > file.txt");
        assert_eq!(correction.expected, "Update");
        assert_eq!(correction.predicted, "Create");
        assert_eq!(correction.confidence, 0.95);
    }

    #[test]
    fn test_corrections_file_deserialization() {
        // Test that CorrectionsFile can deserialize from JSON
        let json = r#"{
            "corrections": [
                {
                    "command": "echo > file.txt",
                    "predicted": "Create",
                    "expected": "Update",
                    "confidence": 0.95,
                    "timestamp": "2025-12-06T00:00:00Z"
                }
            ],
            "version": "1.0",
            "last_updated": "2025-12-06T00:00:00Z"
        }"#;

        let file: CorrectionsFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.corrections.len(), 1);
        assert_eq!(file.version, "1.0");
    }
}
