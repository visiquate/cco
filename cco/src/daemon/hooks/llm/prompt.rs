//! Prompt engineering for CRUD classification
//!
//! Provides prompt construction and response parsing for the
//! embedded LLM classifier.

use crate::daemon::hooks::{CrudClassification, HookError, HookResult};
use std::str::FromStr;

/// Build a CRUD classification prompt for the given command
///
/// Creates a focused, single-shot prompt that asks the model to
/// classify the command as exactly one of: READ, CREATE, UPDATE, DELETE.
///
/// # Arguments
///
/// * `command` - The shell command to classify
///
/// # Returns
///
/// A prompt string suitable for the TinyLLaMA model
///
/// # Example
///
/// ```rust
/// use cco::daemon::hooks::llm::build_crud_prompt;
///
/// let prompt = build_crud_prompt("ls -la");
/// assert!(prompt.contains("READ"));
/// ```
pub fn build_crud_prompt(command: &str) -> String {
    format!(
        r#"Classify this shell command as EXACTLY ONE of: READ, CREATE, UPDATE, or DELETE

Examples:
Command: ls -la
Classification: READ

Command: mkdir newdir
Classification: CREATE

Command: echo test >> file.txt
Classification: UPDATE

Command: rm file.txt
Classification: DELETE

Now classify this command:
Command: {}

Rules:
- READ: Retrieves/displays data, no side effects (ls, cat, grep, git status, ps, find)
- CREATE: Makes new resources (touch, mkdir, git init, docker run, echo > file)
- UPDATE: Modifies existing resources (echo >>, sed -i, git commit, chmod, mv, cp)
- DELETE: Removes resources (rm, rmdir, docker rm, git branch -d, git clean)

Respond with ONLY one word - the classification:"#,
        command.trim()
    )
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
        assert!(prompt.contains("READ"));
        assert!(prompt.contains("CREATE"));
        assert!(prompt.contains("UPDATE"));
        assert!(prompt.contains("DELETE"));
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
}
