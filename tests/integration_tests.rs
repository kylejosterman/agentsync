// Allow expect/unwrap in tests for brevity
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

mod common;

use std::process::Command;

// Note: These tests verify the CLI commands execute without errors.
// More comprehensive integration tests are in cli_tests.rs

#[test]
fn test_init_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "init"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Init should either succeed or fail if already initialized
    let has_output = stdout.contains("Created .agentsync/rules/")
        || stdout.contains("Created agentsync.json")
        || stderr.contains("already initialized");

    assert!(
        has_output,
        "Expected init output, got stdout: {stdout}, stderr: {stderr}"
    );
}

#[test]
fn test_sync_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "sync"])
        .output()
        .expect("Failed to execute command");

    // The sync command should either succeed or fail gracefully
    // It may fail if there's no .agentsync directory, which is expected
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that it's either a sync summary or an expected error
    let has_output = stdout.contains("Syncing from .agentsync/rules/")
        || stdout.contains("✓")
        || stderr.contains("Project not initialized")
        || stderr.contains("agentsync.json");

    assert!(
        has_output,
        "Expected sync output or error, got stdout: {stdout}, stderr: {stderr}"
    );
}

#[test]
fn test_sync_from_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "sync", "--from", "cursor"])
        .output()
        .expect("Failed to execute command");

    // The sync --from command should either succeed or fail gracefully
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that it's either a sync summary or an expected error
    let has_output = stdout.contains("Syncing from cursor")
        || stdout.contains("✓")
        || stderr.contains("Project not initialized")
        || stderr.contains("agentsync.json");

    assert!(
        has_output,
        "Expected sync output or error, got stdout: {stdout}, stderr: {stderr}"
    );
}

#[test]
fn test_add_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "add", "test-rule"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Add should either succeed or fail if not initialized
    let has_output = stdout.contains("Created .agentsync/rules/test-rule.md")
        || stderr.contains("Project not initialized")
        || stderr.contains("already exists");

    assert!(
        has_output,
        "Expected add output, got stdout: {stdout}, stderr: {stderr}"
    );
}
