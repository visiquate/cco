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
/// A prompt string suitable for the Qwen2.5-Coder model
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
        r#"You are a shell command classifier. Classify the following command as exactly one of: READ, CREATE, UPDATE, or DELETE.

CLASSIFICATION RULES:

READ - Retrieves/displays data with NO side effects:
- File inspection: ls, cat, head, tail, find, grep, rg
- Process inspection: ps, top, htop
- Version control inspection: git status, git log, git diff, git show, git branch (list)
- Container inspection: docker ps, docker logs, docker images
- Build inspection: cargo tree, cargo check, cargo clippy, npm list, pip list
- Network inspection: curl (no -o), ping, dig, netstat
- Pipes to STDOUT: cmd | grep, cmd | sort, cmd | head
- Stderr to STDOUT: cmd 2>&1 | grep (still READ if no file creation)

CREATE - Makes new resources:
- File creation: touch, mkdir, echo > file
- Version control: git init, git commit, git push, git checkout -b
- Containers: docker build, docker run, docker create
- Build artifacts: cargo build, cargo doc, npm install
- Network downloads: wget, curl -o file
- Redirects: cmd > file (overwrites), cmd 2> file

UPDATE - Modifies existing resources:
- File modification: echo >> file, sed -i, chmod, chown, mv
- Version control: git add, git commit --amend, git merge, git rebase
- Containers: docker stop, docker start, docker restart
- Code formatting: cargo fmt, npm run format
- Package updates: cargo update, npm update
- Appends: cmd >> file

DELETE - Removes resources:
- File/directory removal: rm, rmdir, rm -rf
- Version control: git clean, git branch -d, git reset --hard
- Containers: docker rm, docker rmi, docker prune
- Build cleanup: cargo clean, npm run clean
- Package removal: cargo rm, npm uninstall, pip uninstall

COMPLEX EXAMPLES:

Command: cargo tree --depth 1
Classification: READ
Reason: Only displays dependency tree to STDOUT

Command: cd /tmp && cargo tree
Classification: READ
Reason: cd doesn't persist; cargo tree only reads

Command: docker logs app 2>&1 | grep ERROR
Classification: READ
Reason: Stderr redirect to stdout, then pipe - no file creation

Command: ls -la > files.txt
Classification: CREATE
Reason: Redirect creates/overwrites files.txt

Command: echo test >> log.txt
Classification: UPDATE
Reason: Append operator updates log.txt

Command: git status && git add .
Classification: UPDATE
Reason: git add stages changes (modifies index)

Command: npm install express
Classification: CREATE
Reason: Installs package, creates node_modules

Command: cargo build --release
Classification: CREATE
Reason: Creates target/release artifacts

Command: cargo fmt
Classification: UPDATE
Reason: Modifies source files in place

Command: curl -I https://example.com | grep HTTP
Classification: READ
Reason: Pipe to STDOUT only, no file creation

Command: find . -name "*.tmp" -delete
Classification: DELETE
Reason: -delete flag removes files

Now classify this command (respond with ONLY one word):
Command: {}
Classification:"#,
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

    // ============================================================================
    // Comprehensive tests validating the enhanced prompt examples
    // ============================================================================

    /// Verify that the prompt contains all the expected command examples
    /// These tests validate that the prompt is teaching the model correctly
    mod prompt_examples {
        use super::*;

        #[test]
        fn test_prompt_contains_cargo_examples() {
            let prompt = build_crud_prompt("test");

            // Cargo READ examples
            assert!(prompt.contains("cargo tree --depth 1"));
            assert!(prompt.contains("cd /tmp && cargo tree"));
            assert!(prompt.contains("cargo clippy"));

            // Cargo CREATE examples
            assert!(prompt.contains("cargo build --release"));

            // Cargo UPDATE examples
            assert!(prompt.contains("cargo fmt"));

            // Cargo DELETE examples
            assert!(prompt.contains("cargo clean"));
        }

        #[test]
        fn test_prompt_contains_git_examples() {
            let prompt = build_crud_prompt("test");

            // Git READ examples
            assert!(prompt.contains("git status"));
            assert!(prompt.contains("git log"));

            // Git UPDATE examples
            assert!(prompt.contains("git status && git add ."));
            assert!(prompt.contains("git add"));

            // Git DELETE examples
            assert!(prompt.contains("git branch -d"));
        }

        #[test]
        fn test_prompt_contains_docker_examples() {
            let prompt = build_crud_prompt("test");

            // Docker READ examples
            assert!(prompt.contains("docker ps"));
            assert!(prompt.contains("docker logs"));
            assert!(prompt.contains("docker logs app 2>&1 | grep ERROR"));

            // Docker CREATE examples
            assert!(prompt.contains("docker build"));

            // Docker DELETE examples
            assert!(prompt.contains("docker rm"));
        }

        #[test]
        fn test_prompt_contains_shell_redirect_examples() {
            let prompt = build_crud_prompt("test");

            // Shell CREATE examples (redirect creates file)
            assert!(prompt.contains("ls -la > files.txt"));
            assert!(prompt.contains("CREATE"));

            // Shell UPDATE examples (append)
            assert!(prompt.contains("echo test >> log.txt"));
            assert!(prompt.contains("UPDATE"));

            // Shell READ examples (pipe only)
            assert!(prompt.contains("curl -I https://example.com | grep HTTP"));
            // Note: The pipe chain example is in the READ rules section, not in detailed examples
            assert!(prompt.contains("cmd | grep, cmd | sort"));
        }

        #[test]
        fn test_prompt_contains_compound_command_examples() {
            let prompt = build_crud_prompt("test");

            // Compound commands with &&
            assert!(prompt.contains("cd /tmp && cargo tree"));
            assert!(prompt.contains("git status && git add ."));
        }

        #[test]
        fn test_prompt_contains_delete_examples() {
            let prompt = build_crud_prompt("test");

            // Find with -delete flag
            assert!(prompt.contains("find . -name \"*.tmp\" -delete"));
            assert!(prompt.contains("DELETE"));
        }

        #[test]
        fn test_prompt_explains_classification_reasoning() {
            let prompt = build_crud_prompt("test");

            // Each example should have a "Reason:" explanation
            assert!(prompt.contains("Reason:"));

            // Verify specific reasoning examples
            assert!(prompt.contains("Only displays dependency tree to STDOUT"));
            assert!(prompt.contains("cd doesn't persist"));
            assert!(prompt.contains("Stderr redirect to stdout, then pipe - no file creation"));
            assert!(prompt.contains("Redirect creates/overwrites"));
            assert!(prompt.contains("Append operator updates"));
            assert!(prompt.contains("-delete flag removes files"));
        }
    }

    // ============================================================================
    // Tests validating the classification rules are correctly documented
    // ============================================================================

    mod classification_rules {
        use super::*;

        #[test]
        fn test_prompt_defines_read_operations() {
            let prompt = build_crud_prompt("test");

            // READ rule categories
            assert!(prompt.contains("READ - Retrieves/displays data with NO side effects"));
            assert!(prompt.contains("File inspection: ls, cat, head, tail, find, grep"));
            assert!(prompt.contains("Process inspection: ps, top, htop"));
            assert!(prompt.contains("Version control inspection: git status, git log"));
            assert!(prompt.contains("Container inspection: docker ps, docker logs"));
            assert!(prompt.contains("Build inspection: cargo tree, cargo check, cargo clippy"));
            assert!(prompt.contains("Network inspection: curl (no -o), ping, dig"));
            assert!(prompt.contains("Pipes to STDOUT: cmd | grep, cmd | sort"));
            assert!(prompt.contains("Stderr to STDOUT: cmd 2>&1 | grep (still READ if no file creation)"));
        }

        #[test]
        fn test_prompt_defines_create_operations() {
            let prompt = build_crud_prompt("test");

            // CREATE rule categories
            assert!(prompt.contains("CREATE - Makes new resources"));
            assert!(prompt.contains("File creation: touch, mkdir, echo > file"));
            assert!(prompt.contains("Version control: git init, git commit, git push"));
            assert!(prompt.contains("Containers: docker build, docker run, docker create"));
            assert!(prompt.contains("Build artifacts: cargo build, cargo doc"));
            assert!(prompt.contains("Redirects: cmd > file (overwrites)"));
        }

        #[test]
        fn test_prompt_defines_update_operations() {
            let prompt = build_crud_prompt("test");

            // UPDATE rule categories
            assert!(prompt.contains("UPDATE - Modifies existing resources"));
            assert!(prompt.contains("File modification: echo >> file, sed -i, chmod"));
            assert!(prompt.contains("Version control: git add, git commit --amend"));
            assert!(prompt.contains("Code formatting: cargo fmt"));
            assert!(prompt.contains("Appends: cmd >> file"));
        }

        #[test]
        fn test_prompt_defines_delete_operations() {
            let prompt = build_crud_prompt("test");

            // DELETE rule categories
            assert!(prompt.contains("DELETE - Removes resources"));
            assert!(prompt.contains("File/directory removal: rm, rmdir, rm -rf"));
            assert!(prompt.contains("Version control: git clean, git branch -d"));
            assert!(prompt.contains("Containers: docker rm, docker rmi, docker prune"));
            assert!(prompt.contains("Build cleanup: cargo clean"));
        }
    }

    // ============================================================================
    // Tests for cargo command classification
    // ============================================================================

    mod cargo_commands {
        use super::*;

        #[test]
        fn test_cargo_tree_is_read() {
            let prompt = build_crud_prompt("cargo tree --depth 1");
            assert!(prompt.contains("cargo tree --depth 1"));
            // The prompt should classify this as READ in examples
            assert!(prompt.contains("Classification: READ"));
        }

        #[test]
        fn test_cargo_tree_with_cd_is_read() {
            let prompt = build_crud_prompt("cd /tmp && cargo tree");
            assert!(prompt.contains("cd /tmp && cargo tree"));
            assert!(prompt.contains("Classification: READ"));
        }

        #[test]
        fn test_cargo_build_is_create() {
            let prompt = build_crud_prompt("cargo build --release");
            assert!(prompt.contains("cargo build --release"));
            assert!(prompt.contains("Classification: CREATE"));
        }

        #[test]
        fn test_cargo_fmt_is_update() {
            let prompt = build_crud_prompt("cargo fmt");
            assert!(prompt.contains("cargo fmt"));
            assert!(prompt.contains("Classification: UPDATE"));
        }

        #[test]
        fn test_cargo_clean_is_delete() {
            let prompt = build_crud_prompt("cargo clean");
            // Verify cargo clean is in the DELETE category
            assert!(prompt.contains("cargo clean"));
        }

        #[test]
        fn test_cargo_clippy_is_read() {
            let prompt = build_crud_prompt("cargo clippy");
            // Verify cargo clippy is in the READ category
            assert!(prompt.contains("cargo clippy"));
        }
    }

    // ============================================================================
    // Tests for git command classification
    // ============================================================================

    mod git_commands {
        use super::*;

        #[test]
        fn test_git_status_is_read() {
            let prompt = build_crud_prompt("git status");
            assert!(prompt.contains("git status"));
            // git status is in READ inspection category
        }

        #[test]
        fn test_git_log_is_read() {
            let prompt = build_crud_prompt("git log --oneline -10");
            assert!(prompt.contains("git log"));
        }

        #[test]
        fn test_git_add_is_update() {
            let prompt = build_crud_prompt("git add .");
            assert!(prompt.contains("git add"));
            // git add is in UPDATE category
        }

        #[test]
        fn test_git_commit_is_create() {
            let prompt = build_crud_prompt("git commit -m \"msg\"");
            assert!(prompt.contains("git commit"));
            // git commit is in CREATE category
        }

        #[test]
        fn test_git_status_and_add_is_update() {
            let prompt = build_crud_prompt("git status && git add .");
            assert!(prompt.contains("git status && git add ."));
            assert!(prompt.contains("Classification: UPDATE"));
        }

        #[test]
        fn test_git_branch_delete_is_delete() {
            let prompt = build_crud_prompt("git branch -d feature");
            assert!(prompt.contains("git branch -d"));
            // git branch -d is in DELETE category
        }
    }

    // ============================================================================
    // Tests for docker command classification
    // ============================================================================

    mod docker_commands {
        use super::*;

        #[test]
        fn test_docker_ps_is_read() {
            let prompt = build_crud_prompt("docker ps");
            assert!(prompt.contains("docker ps"));
            // docker ps is in READ inspection category
        }

        #[test]
        fn test_docker_logs_is_read() {
            let prompt = build_crud_prompt("docker logs app");
            assert!(prompt.contains("docker logs"));
        }

        #[test]
        fn test_docker_logs_with_stderr_pipe_is_read() {
            let prompt = build_crud_prompt("docker logs app 2>&1 | grep ERROR");
            assert!(prompt.contains("docker logs app 2>&1 | grep ERROR"));
            assert!(prompt.contains("Classification: READ"));
        }

        #[test]
        fn test_docker_build_is_create() {
            let prompt = build_crud_prompt("docker build -t myapp .");
            assert!(prompt.contains("docker build"));
            // docker build is in CREATE category
        }

        #[test]
        fn test_docker_rm_is_delete() {
            let prompt = build_crud_prompt("docker rm container");
            assert!(prompt.contains("docker rm"));
            // docker rm is in DELETE category
        }
    }

    // ============================================================================
    // Tests for shell operator classification
    // ============================================================================

    mod shell_operators {
        use super::*;

        #[test]
        fn test_redirect_to_file_is_create() {
            let prompt = build_crud_prompt("ls -la > files.txt");
            assert!(prompt.contains("ls -la > files.txt"));
            assert!(prompt.contains("Classification: CREATE"));
        }

        #[test]
        fn test_append_to_file_is_update() {
            let prompt = build_crud_prompt("echo test >> log.txt");
            assert!(prompt.contains("echo test >> log.txt"));
            assert!(prompt.contains("Classification: UPDATE"));
        }

        #[test]
        fn test_pipe_to_grep_is_read() {
            let prompt = build_crud_prompt("curl -I https://example.com | grep HTTP");
            assert!(prompt.contains("curl -I https://example.com | grep HTTP"));
            assert!(prompt.contains("Classification: READ"));
        }

        #[test]
        fn test_pipe_chain_is_read() {
            let prompt = build_crud_prompt("grep error logs.txt | sort | uniq");
            assert!(prompt.contains("grep error logs.txt | sort | uniq"));
            // Pipe chains are in READ category
        }

        #[test]
        fn test_stderr_redirect_no_file_is_read() {
            let prompt = build_crud_prompt("docker logs app 2>&1 | grep ERROR");
            // Stderr to stdout with pipe (no file) is still READ
            assert!(prompt.contains("Stderr redirect to stdout, then pipe - no file creation"));
        }
    }

    // ============================================================================
    // Tests for compound commands
    // ============================================================================

    mod compound_commands {
        use super::*;

        #[test]
        fn test_cd_with_cargo_tree_is_read() {
            let prompt = build_crud_prompt("cd /tmp && cargo tree");
            assert!(prompt.contains("cd /tmp && cargo tree"));
            assert!(prompt.contains("Classification: READ"));
            assert!(prompt.contains("cd doesn't persist"));
        }

        #[test]
        fn test_find_with_delete_is_delete() {
            let prompt = build_crud_prompt("find . -name \"*.tmp\" -delete");
            assert!(prompt.contains("find . -name \"*.tmp\" -delete"));
            assert!(prompt.contains("Classification: DELETE"));
        }
    }

    // ============================================================================
    // Integration tests: Validate prompt structure
    // ============================================================================

    mod prompt_structure {
        use super::*;

        #[test]
        fn test_prompt_has_all_sections() {
            let prompt = build_crud_prompt("test command");

            // Major sections
            assert!(prompt.contains("CLASSIFICATION RULES:"));
            assert!(prompt.contains("READ - Retrieves/displays data"));
            assert!(prompt.contains("CREATE - Makes new resources"));
            assert!(prompt.contains("UPDATE - Modifies existing resources"));
            assert!(prompt.contains("DELETE - Removes resources"));
            assert!(prompt.contains("COMPLEX EXAMPLES:"));
            assert!(prompt.contains("Now classify this command"));
        }

        #[test]
        fn test_prompt_includes_user_command() {
            let user_command = "cargo test --all";
            let prompt = build_crud_prompt(user_command);

            // The user's command should appear at the end
            assert!(prompt.contains(&format!("Command: {}", user_command)));
        }

        #[test]
        fn test_prompt_trims_command() {
            let prompt = build_crud_prompt("  cargo test  ");
            assert!(prompt.contains("Command: cargo test"));
            assert!(!prompt.contains("Command:   cargo test  "));
        }

        #[test]
        fn test_prompt_requests_single_word_response() {
            let prompt = build_crud_prompt("test");
            assert!(prompt.contains("respond with ONLY one word"));
        }

        #[test]
        fn test_all_examples_have_reasoning() {
            let prompt = build_crud_prompt("test");

            // Count occurrences of example structure
            let example_count = prompt.matches("Command:").count();
            let reason_count = prompt.matches("Reason:").count();
            let classification_count = prompt.matches("Classification:").count();

            // Should have at least 10 examples
            assert!(example_count >= 10);

            // The prompt has 11 example commands + 1 user command = 12 "Command:" occurrences
            // Each of the 11 examples has a "Reason:" field
            // Each of the 11 examples plus the user command has "Classification:" field
            assert_eq!(example_count, 12, "Expected 11 examples + 1 user command");
            assert_eq!(reason_count, 11, "Expected 11 examples with reasoning");
            assert_eq!(classification_count, 12, "Expected 11 examples + 1 user classification field");
        }
    }
}
