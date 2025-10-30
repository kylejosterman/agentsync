//! Integration tests for file system operations

// Allow expect/unwrap in tests for brevity
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

mod common;

use agentsync::fs::{
    Tool, discover_rules, ensure_directory, extract_rule_name, find_project_root, read_rule_file,
    rule_path, validate_rule_name, write_rule_file,
};
use common::TestContext;
use fs_err as fs;
use std::path::Path;

#[test]
fn test_discover_cursor_fixtures() {
    let ctx = TestContext::new();

    // Copy cursor fixtures to temp location
    let copied = ctx.copy_fixtures(Tool::Cursor);

    if !copied.is_empty() {
        let rules = discover_rules(ctx.root(), Tool::Cursor).unwrap();
        assert!(!rules.is_empty(), "Should discover cursor fixture files");

        // Verify we can read each discovered file
        for rule_path in rules {
            let content = read_rule_file(&rule_path).unwrap();
            assert!(!content.is_empty(), "Rule file should have content");
        }
    }
}

#[test]
fn test_discover_copilot_fixtures() {
    let ctx = TestContext::new();

    let copied = ctx.copy_fixtures(Tool::Copilot);

    if !copied.is_empty() {
        let rules = discover_rules(ctx.root(), Tool::Copilot).unwrap();
        assert!(!rules.is_empty(), "Should discover copilot fixture files");
    }
}

#[test]
fn test_discover_windsurf_fixtures() {
    let ctx = TestContext::new();

    let copied = ctx.copy_fixtures(Tool::Windsurf);

    if !copied.is_empty() {
        let rules = discover_rules(ctx.root(), Tool::Windsurf).unwrap();
        assert!(!rules.is_empty(), "Should discover windsurf fixture files");
    }
}

#[test]
fn test_discover_agentsync_fixtures() {
    let ctx = TestContext::new();

    let copied = ctx.copy_fixtures(Tool::AgentSync);

    if !copied.is_empty() {
        let rules = discover_rules(ctx.root(), Tool::AgentSync).unwrap();
        assert!(!rules.is_empty(), "Should discover agentsync fixture files");
    }
}

#[test]
fn test_roundtrip_write_and_read() {
    let ctx = TestContext::new();

    let content = r"---
description: Test rule
alwaysApply: true
---

# Test Rule

This is test content.
";

    // Write a cursor rule
    let cursor_path = rule_path(ctx.root(), Tool::Cursor, "test-rule").unwrap();
    write_rule_file(&cursor_path, content).unwrap();

    // Verify it was written
    assert!(cursor_path.exists());

    // Read it back
    let read_content = read_rule_file(&cursor_path).unwrap();
    assert_eq!(read_content, content);

    // Verify the directory structure
    assert!(ctx.path(".cursor/rules").exists());
}

#[test]
fn test_write_multiple_tools() {
    let ctx = TestContext::new();

    let content = "# Rule content";

    // Write to all tool directories
    for tool in [Tool::Cursor, Tool::Copilot, Tool::Windsurf, Tool::AgentSync] {
        let path = rule_path(ctx.root(), tool, "shared-rule").unwrap();
        write_rule_file(&path, content).unwrap();
        assert!(path.exists());
    }

    // Verify each tool has its own file with correct extension
    ctx.assert_rule_exists(Tool::Cursor, "shared-rule");
    ctx.assert_rule_exists(Tool::Copilot, "shared-rule");
    ctx.assert_rule_exists(Tool::Windsurf, "shared-rule");
    ctx.assert_rule_exists(Tool::AgentSync, "shared-rule");
}

#[test]
fn test_extract_rule_names_from_fixtures() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // Test with cursor fixtures
    let cursor_fixtures = project_root.join("tests/fixtures/cursor");
    if cursor_fixtures.exists() {
        for entry in fs::read_dir(&cursor_fixtures).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("mdc") {
                let rule_name = extract_rule_name(&path).unwrap();
                assert!(!rule_name.is_empty());
                // Verify it's a valid rule name
                assert!(validate_rule_name(&rule_name).is_ok());
            }
        }
    }
}

#[test]
fn test_ensure_nested_directories() {
    let ctx = TestContext::new();

    // Ensure deeply nested directory
    let deep_path = ctx.path("level1/level2/level3/level4");
    ensure_directory(&deep_path).unwrap();
    assert!(deep_path.exists());
    assert!(deep_path.is_dir());
}

#[test]
fn test_discover_rules_ignores_wrong_extensions() {
    let ctx = TestContext::new();

    // Create cursor directory with mixed extensions
    let cursor_dir = ctx.path(".cursor/rules");
    fs::create_dir_all(&cursor_dir).unwrap();

    // Create files with various extensions
    fs::write(cursor_dir.join("rule1.mdc"), "correct").unwrap();
    fs::write(cursor_dir.join("rule2.mdc"), "correct").unwrap();
    fs::write(cursor_dir.join("rule3.md"), "wrong extension").unwrap();
    fs::write(cursor_dir.join("rule4.txt"), "wrong extension").unwrap();
    fs::write(cursor_dir.join("README.mdc"), "correct").unwrap();

    let rules = discover_rules(ctx.root(), Tool::Cursor).unwrap();

    // Should only find .mdc files
    assert_eq!(rules.len(), 3);
    for rule in &rules {
        assert_eq!(rule.extension().unwrap(), "mdc");
    }
}

#[test]
fn test_rule_path_construction() {
    let ctx = TestContext::new();
    let project_root = ctx.root();

    // Test each tool
    let cursor_path = rule_path(project_root, Tool::Cursor, "python-dev").unwrap();
    assert!(
        cursor_path
            .to_str()
            .unwrap()
            .ends_with(".cursor/rules/python-dev.mdc")
    );

    let copilot_path = rule_path(project_root, Tool::Copilot, "react-rules").unwrap();
    assert!(
        copilot_path
            .to_str()
            .unwrap()
            .ends_with(".github/instructions/react-rules.instructions.md")
    );

    let windsurf_path = rule_path(project_root, Tool::Windsurf, "rust-dev").unwrap();
    assert!(
        windsurf_path
            .to_str()
            .unwrap()
            .ends_with(".windsurf/rules/rust-dev.md")
    );

    let agentsync_path = rule_path(project_root, Tool::AgentSync, "general").unwrap();
    assert!(
        agentsync_path
            .to_str()
            .unwrap()
            .ends_with(".agentsync/rules/general.md")
    );
}

#[test]
fn test_find_project_root_with_config() {
    let ctx = TestContext::new().init_project();

    // Change to that directory and find root
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(ctx.root()).unwrap();

    let found_root = find_project_root().unwrap();

    // Canonicalize both paths to handle symlinks (e.g., /var -> /private/var on macOS)
    let canonical_found = found_root.canonicalize().unwrap();
    let canonical_expected = ctx.root().canonicalize().unwrap();
    assert_eq!(canonical_found, canonical_expected);

    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_write_overwrites_existing_file() {
    let ctx = TestContext::new();

    let path = rule_path(ctx.root(), Tool::Cursor, "test-rule").unwrap();

    // Write initial content
    write_rule_file(&path, "initial content").unwrap();
    assert_eq!(read_rule_file(&path).unwrap(), "initial content");

    // Overwrite with new content
    write_rule_file(&path, "updated content").unwrap();
    assert_eq!(read_rule_file(&path).unwrap(), "updated content");
}
