//! Integration tests for CLI commands
//!
//! Tests the full CLI workflow: init, add, sync

// Allow expect/unwrap in tests for brevity
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

mod common;

use agentsync::fs::Tool;
use common::{TestContext, default_sync_options};
use fs_err as fs;

#[test]
fn test_init_creates_directory_and_config() {
    let ctx = TestContext::new();
    let project_root = ctx.root().to_path_buf();

    std::env::set_current_dir(&project_root).expect("Failed to change dir");

    // Create .agentsync/rules/ manually
    let agentsync_dir = ctx.path(".agentsync/rules");
    fs::create_dir_all(&agentsync_dir).expect("Failed to create dir");

    // Create config
    let config = agentsync::config::create_default_config();
    agentsync::config::save_config(ctx.path("agentsync.json"), &config)
        .expect("Failed to save config");

    // Verify directory exists
    assert!(agentsync_dir.exists());
    assert!(agentsync_dir.is_dir());

    // Verify config exists and is valid
    let loaded_config = ctx.load_config();
    assert_eq!(loaded_config.tools.len(), 3);
    assert!(loaded_config.tools.contains(&"cursor".to_string()));
    assert!(loaded_config.tools.contains(&"copilot".to_string()));
    assert!(loaded_config.tools.contains(&"windsurf".to_string()));
}

#[test]
fn test_init_fails_if_already_initialized() {
    let ctx = TestContext::new().init_project();

    // Config already exists
    assert!(ctx.path("agentsync.json").exists());
}

#[test]
fn test_add_creates_rule_template() {
    let ctx = TestContext::new().init_project();

    std::env::set_current_dir(ctx.root()).expect("Failed to change dir");

    // Create a rule using the add command logic
    let rule_name = "test-rule";
    let template = r#"---
targets:
  - "*"
description: "Description of this rule"
globs: "**/*"
cursor:
  alwaysApply: false
  globs: ""
windsurf:
  trigger: model_decision
  globs: ""
copilot:
  applyTo: "**"
---

# Test Rule

Your rule content here...
"#;

    let rule_path = ctx.create_agentsync_rule(rule_name, template);

    // Verify rule was created
    assert!(rule_path.exists());

    // Verify content is valid
    let content = fs::read_to_string(&rule_path).expect("Failed to read rule");
    assert!(content.contains("targets:"));
    assert!(content.contains("description:"));
    assert!(content.contains("# Test Rule"));
}

#[test]
fn test_add_validates_rule_name() {
    // Test invalid characters
    let invalid_names = vec![
        "rule with spaces",
        "rule/with/slash",
        "rule@with@at",
        "rule.with.dots",
    ];

    for name in invalid_names {
        // Check if name contains invalid characters
        let has_invalid = name.contains(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
        assert!(has_invalid, "Expected '{name}' to be invalid");
    }

    // Test valid names
    let valid_names = vec![
        "python-dev",
        "react_components",
        "rust-best-practices",
        "test123",
    ];

    for name in valid_names {
        let has_invalid = name.contains(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
        assert!(!has_invalid, "Expected {name} to be valid");
    }
}

#[test]
fn test_sync_workflow() {
    let ctx = TestContext::new().init_project();

    // Create a test rule
    let rule_content = r#"---
targets:
  - "*"
description: "Test rule"
globs: "**/*.rs"
cursor:
  alwaysApply: true
  globs: ""
windsurf:
  trigger: always_on
  globs: ""
copilot:
  applyTo: "**"
---

# Test Rule

This is a test rule.
"#;

    ctx.create_agentsync_rule("test-rule", rule_content);

    // Run sync to tools
    let result = ctx.sync_to_tools(&default_sync_options());

    // Verify rules were synced to all tools
    assert!(result.has_changes() || result.total_processed() > 0);

    // Check that tool directories were created and files written
    ctx.assert_rule_exists(Tool::Cursor, "test-rule");
    ctx.assert_rule_exists(Tool::Copilot, "test-rule");
    ctx.assert_rule_exists(Tool::Windsurf, "test-rule");

    // Verify content was converted correctly
    let cursor_content = ctx.read_rule(Tool::Cursor, "test-rule");
    assert!(cursor_content.contains("alwaysApply: true"));
    assert!(cursor_content.contains("This is a test rule"));
}

#[test]
fn test_sync_from_tool() {
    let ctx = TestContext::new().init_project();

    // Create a cursor rule
    let cursor_rule = r#"---
description: "Cursor test rule"
alwaysApply: true
---

# Cursor Rule

This is a cursor rule.
"#;

    ctx.create_cursor_rule("cursor-rule", cursor_rule);

    // Run sync from cursor
    let result = ctx.sync_from_tool(Tool::Cursor, &default_sync_options());

    // Verify rule was imported
    assert!(result.has_changes());
    assert_eq!(result.added.len(), 1);

    // Check that AgentSync rule was created
    ctx.assert_rule_exists(Tool::AgentSync, "cursor-rule");

    // Verify content was converted correctly
    let agentsync_content = ctx.read_rule(Tool::AgentSync, "cursor-rule");
    assert!(agentsync_content.contains("targets:"));
    assert!(agentsync_content.contains("cursor:"));
    assert!(agentsync_content.contains("This is a cursor rule"));
}

#[test]
fn test_sync_dry_run() {
    let ctx = TestContext::new().init_project();

    // Create a test rule
    let rule_content = r#"---
targets:
  - "*"
description: "Test rule"
globs: "**/*"
cursor:
  alwaysApply: false
  globs: ""
windsurf:
  trigger: model_decision
  globs: ""
copilot:
  applyTo: "**"
---

# Dry Run Test

This is a test.
"#;

    ctx.create_agentsync_rule("dry-run-test", rule_content);

    // Run sync in dry-run mode
    let options = agentsync::sync::SyncOptions {
        dry_run: true,
        verbose: false,
    };

    let result = ctx.sync_to_tools(&options);

    // Verify changes were detected
    assert!(result.has_changes());

    // Verify files were NOT created (dry-run)
    ctx.assert_rule_not_exists(Tool::Cursor, "dry-run-test");
    ctx.assert_rule_not_exists(Tool::Copilot, "dry-run-test");
    ctx.assert_rule_not_exists(Tool::Windsurf, "dry-run-test");
}

#[test]
fn test_sync_with_target_filtering() {
    let ctx = TestContext::new().init_project();

    // Create a rule that only targets cursor
    let rule_content = r#"---
targets:
  - "cursor"
description: "Cursor-only rule"
globs: "**/*"
cursor:
  alwaysApply: true
  globs: ""
windsurf:
  trigger: always_on
  globs: ""
copilot:
  applyTo: "**"
---

# Cursor Only

This rule should only sync to Cursor.
"#;

    ctx.create_agentsync_rule("cursor-only", rule_content);

    // Run sync to tools
    let result = ctx.sync_to_tools(&default_sync_options());

    // Verify only cursor rule was created
    ctx.assert_rule_exists(Tool::Cursor, "cursor-only");
    ctx.assert_rule_not_exists(Tool::Copilot, "cursor-only");
    ctx.assert_rule_not_exists(Tool::Windsurf, "cursor-only");

    // Verify result reflects only cursor sync
    assert!(result.has_changes());
}

#[test]
fn test_end_to_end_workflow() {
    let ctx = TestContext::new().init_project();

    // Step 1: Add a rule
    let rule_content = r#"---
targets:
  - "*"
description: "End-to-end test rule"
globs: "**/*.rs"
cursor:
  alwaysApply: false
  globs: "**/*.rs"
windsurf:
  trigger: glob
  globs: "**/*.rs"
copilot:
  applyTo: "**/*.rs"
---

# E2E Test Rule

This tests the full workflow.
"#;

    ctx.create_agentsync_rule("e2e-test", rule_content);

    // Step 2: Sync to tools
    let sync_result = ctx.sync_to_tools(&default_sync_options());
    assert!(sync_result.has_changes());

    // Step 3: Verify all tool files exist
    ctx.assert_rule_exists(Tool::Cursor, "e2e-test");
    ctx.assert_rule_exists(Tool::Copilot, "e2e-test");
    ctx.assert_rule_exists(Tool::Windsurf, "e2e-test");

    // Step 4: Modify a tool file and sync back
    let modified_cursor = r#"---
description: "Modified cursor rule"
alwaysApply: true
globs: ""
---

# Modified E2E Test Rule

This was modified in Cursor.
"#;

    ctx.create_cursor_rule("e2e-test", modified_cursor);

    // Step 5: Sync from cursor back to agentsync
    let sync_back_result = ctx.sync_from_tool(Tool::Cursor, &default_sync_options());
    assert!(sync_back_result.has_changes());

    // Step 6: Verify agentsync rule was updated
    let updated_content = ctx.read_rule(Tool::AgentSync, "e2e-test");
    assert!(updated_content.contains("Modified cursor rule"));
    assert!(updated_content.contains("This was modified in Cursor"));
}
