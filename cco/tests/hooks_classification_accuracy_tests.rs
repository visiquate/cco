//! Comprehensive tests for CRUD classification accuracy
//!
//! Tests classification accuracy across all CRUD categories with a wide
//! variety of real-world commands. Target: 15+ classification tests.
//!
//! Tests organized by CRUD type:
//! - READ operations (8 tests)
//! - CREATE operations (8 tests)
//! - UPDATE operations (7 tests)
//! - DELETE operations (7 tests)
//!
//! Run with: cargo test hooks_classification_accuracy

mod hooks_test_helpers;

use hooks_test_helpers::*;

// =============================================================================
// SECTION 1: READ Classification Tests (8 tests)
// =============================================================================

#[tokio::test]
#[ignore] // Remove when /api/classify is implemented
async fn test_classify_ls_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("ls -la").await.unwrap();
    assert_classification(&response, "READ", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_cat_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("cat file.txt").await.unwrap();
    assert_classification(&response, "READ", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_grep_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("grep -r 'pattern' .").await.unwrap();
    assert_classification(&response, "READ", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_git_status() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("git status").await.unwrap();
    assert_classification(&response, "READ", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_ps_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("ps aux").await.unwrap();
    assert_classification(&response, "READ", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_find_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("find . -name '*.rs'").await.unwrap();
    assert_classification(&response, "READ", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_docker_ps() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("docker ps -a").await.unwrap();
    assert_classification(&response, "READ", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_head_tail_commands() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("head -20 log.txt").await.unwrap();
    assert_classification(&response, "READ", 0.8);

    let response = daemon
        .client
        .classify("tail -f application.log")
        .await
        .unwrap();
    assert_classification(&response, "READ", 0.8);
}

// =============================================================================
// SECTION 2: CREATE Classification Tests (8 tests)
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_classify_touch_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("touch newfile.txt").await.unwrap();
    assert_classification(&response, "CREATE", 0.7);
}

#[tokio::test]
#[ignore]
async fn test_classify_mkdir_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("mkdir -p path/to/dir")
        .await
        .unwrap();
    assert_classification(&response, "CREATE", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_docker_run() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("docker run -d nginx").await.unwrap();
    assert_classification(&response, "CREATE", 0.7);
}

#[tokio::test]
#[ignore]
async fn test_classify_git_init() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("git init").await.unwrap();
    assert_classification(&response, "CREATE", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_output_redirect_create() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("echo 'hello' > output.txt")
        .await
        .unwrap();
    assert_classification(&response, "CREATE", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_npm_init() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("npm init -y").await.unwrap();
    assert_classification(&response, "CREATE", 0.7);
}

#[tokio::test]
#[ignore]
async fn test_classify_cargo_new() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("cargo new my-project")
        .await
        .unwrap();
    assert_classification(&response, "CREATE", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_git_branch_create() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("git branch new-feature")
        .await
        .unwrap();
    assert_classification(&response, "CREATE", 0.7);
}

// =============================================================================
// SECTION 3: UPDATE Classification Tests (7 tests)
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_classify_echo_append() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("echo 'data' >> file.txt")
        .await
        .unwrap();
    assert_classification(&response, "UPDATE", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_git_commit() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("git commit -m 'Update README'")
        .await
        .unwrap();
    assert_classification(&response, "UPDATE", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_chmod_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("chmod +x script.sh").await.unwrap();
    assert_classification(&response, "UPDATE", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_sed_inplace() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("sed -i 's/old/new/g' file.txt")
        .await
        .unwrap();
    assert_classification(&response, "UPDATE", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_git_add() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("git add .").await.unwrap();
    assert_classification(&response, "UPDATE", 0.7);
}

#[tokio::test]
#[ignore]
async fn test_classify_mv_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("mv oldname.txt newname.txt")
        .await
        .unwrap();
    assert_classification(&response, "UPDATE", 0.7);
}

#[tokio::test]
#[ignore]
async fn test_classify_chown_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("chown user:group file.txt")
        .await
        .unwrap();
    assert_classification(&response, "UPDATE", 0.7);
}

// =============================================================================
// SECTION 4: DELETE Classification Tests (7 tests)
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_classify_rm_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("rm file.txt").await.unwrap();
    assert_classification(&response, "DELETE", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_rm_rf_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("rm -rf directory/").await.unwrap();
    assert_classification(&response, "DELETE", 0.9);
}

#[tokio::test]
#[ignore]
async fn test_classify_rmdir_command() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("rmdir empty_directory")
        .await
        .unwrap();
    assert_classification(&response, "DELETE", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_docker_rm() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("docker rm container_name")
        .await
        .unwrap();
    assert_classification(&response, "DELETE", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_git_branch_delete() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("git branch -d feature-branch")
        .await
        .unwrap();
    assert_classification(&response, "DELETE", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_git_clean() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("git clean -fd").await.unwrap();
    assert_classification(&response, "DELETE", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_npm_uninstall() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("npm uninstall package-name")
        .await
        .unwrap();
    assert_classification(&response, "DELETE", 0.7);
}

// =============================================================================
// SECTION 5: Complex Command Tests (5 tests)
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_classify_piped_read_commands() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Multiple read operations piped together
    let response = daemon
        .client
        .classify("cat file.txt | grep pattern | sort | uniq")
        .await
        .unwrap();
    assert_classification(&response, "READ", 0.7);
}

#[tokio::test]
#[ignore]
async fn test_classify_command_with_background() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Background execution doesn't change classification
    let response = daemon
        .client
        .classify("docker run -d nginx &")
        .await
        .unwrap();
    assert_classification(&response, "CREATE", 0.6);
}

#[tokio::test]
#[ignore]
async fn test_classify_compound_command_and() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Should classify based on most destructive operation
    let response = daemon
        .client
        .classify("mkdir test && cd test && rm -rf *")
        .await
        .unwrap();
    // This contains DELETE, which should be detected
    assert_classification(&response, "DELETE", 0.6);
}

#[tokio::test]
#[ignore]
async fn test_classify_command_substitution() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon.client.classify("echo $(ls -la)").await.unwrap();
    // Overall creates output, but inner command is read
    assert_classification(&response, "CREATE", 0.5);
}

#[tokio::test]
#[ignore]
async fn test_classify_here_document() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    let response = daemon
        .client
        .classify("cat > file.txt << EOF\nContent\nEOF")
        .await
        .unwrap();
    assert_classification(&response, "CREATE", 0.7);
}

// =============================================================================
// SECTION 6: Edge Cases and Ambiguous Commands (5 tests)
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_classify_git_log_read() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("git log --oneline").await.unwrap();
    assert_classification(&response, "READ", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_git_diff_read() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon.client.classify("git diff HEAD~1").await.unwrap();
    assert_classification(&response, "READ", 0.8);
}

#[tokio::test]
#[ignore]
async fn test_classify_docker_build_create() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();
    let response = daemon
        .client
        .classify("docker build -t myapp:latest .")
        .await
        .unwrap();
    assert_classification(&response, "CREATE", 0.7);
}

#[tokio::test]
#[ignore]
async fn test_classify_curl_read() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Curl without output redirect is READ
    let response = daemon
        .client
        .classify("curl https://example.com")
        .await
        .unwrap();
    assert_classification(&response, "READ", 0.7);
}

#[tokio::test]
#[ignore]
async fn test_classify_curl_download_create() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Curl with -o creates a file
    let response = daemon
        .client
        .classify("curl -o file.zip https://example.com/file.zip")
        .await
        .unwrap();
    assert_classification(&response, "CREATE", 0.7);
}

// =============================================================================
// SECTION 7: Confidence Score Validation (3 tests)
// =============================================================================

#[tokio::test]
#[ignore]
async fn test_high_confidence_for_obvious_commands() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Very obvious commands should have high confidence
    let obvious_commands = vec![
        ("ls", "READ"),
        ("mkdir test", "CREATE"),
        ("rm file", "DELETE"),
    ];

    for (cmd, expected_class) in obvious_commands {
        let response = daemon.client.classify(cmd).await.unwrap();
        assert_eq!(response.classification.to_uppercase(), expected_class);
        assert!(
            response.confidence > 0.8,
            "Command '{}' should have high confidence",
            cmd
        );
    }
}

#[tokio::test]
#[ignore]
async fn test_lower_confidence_for_ambiguous_commands() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // More complex commands may have lower confidence
    let complex_cmd = "git stash && git pull && git stash pop";
    let response = daemon.client.classify(complex_cmd).await.unwrap();

    // Should still classify, but confidence may be lower
    assert!(!response.classification.is_empty());
    assert!(response.confidence >= 0.0 && response.confidence <= 1.0);
}

#[tokio::test]
#[ignore]
async fn test_confidence_score_consistency() {
    let daemon = TestDaemon::with_hooks_enabled().await.unwrap();

    // Same command should get consistent confidence scores
    let cmd = "cat file.txt";

    let response1 = daemon.client.classify(cmd).await.unwrap();
    let response2 = daemon.client.classify(cmd).await.unwrap();

    // Confidence should be very similar (within 0.1)
    let diff = (response1.confidence - response2.confidence).abs();
    assert!(diff < 0.1, "Confidence scores should be consistent");
}
