//! Integration tests for sync functionality

// Allow expect/unwrap in tests for brevity
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

mod common;

use agentsync::fs::Tool;
use agentsync::sync::SyncOptions;
use common::{TestContext, assert_sync_result, default_sync_options, simple_agentsync_rule};

#[test]
fn test_sync_to_tools_basic() {
    let ctx = TestContext::new().init_project();

    ctx.create_agentsync_rule("test-rule", &simple_agentsync_rule("Test rule", "**/*.rs"));

    let result = ctx.sync_to_tools(&default_sync_options());

    assert_sync_result(&result, 3, 0, 0, 0); // cursor, copilot, windsurf
    ctx.assert_rule_exists(Tool::Cursor, "test-rule");
    ctx.assert_rule_exists(Tool::Copilot, "test-rule");
    ctx.assert_rule_exists(Tool::Windsurf, "test-rule");
}

#[test]
fn test_sync_to_tools_targeted() {
    let ctx = TestContext::new().init_project();

    // Create an AgentSync rule that only targets cursor
    let rule_content = r#"---
targets: ["cursor"]
description: "Cursor-only rule"
globs: "**/*.rs"
cursor:
  alwaysApply: true
  globs: ""
---

# Cursor Only

This rule only targets cursor.
"#;
    ctx.create_agentsync_rule("cursor-only", rule_content);

    let result = ctx.sync_to_tools(&default_sync_options());

    assert_sync_result(&result, 1, 0, 0, 0);
    assert!(
        result
            .added
            .iter()
            .any(|r| r.contains("cursor-only") && r.contains("cursor"))
    );

    ctx.assert_rule_exists(Tool::Cursor, "cursor-only");
    ctx.assert_rule_not_exists(Tool::Copilot, "cursor-only");
    ctx.assert_rule_not_exists(Tool::Windsurf, "cursor-only");
}

#[test]
fn test_sync_to_tools_dry_run() {
    let ctx = TestContext::new().init_project_with_tools(&["cursor"]);

    let rule_content = r#"---
targets: ["*"]
description: "Test rule"
globs: "**/*"
---

# Test Rule

Dry run test.
"#;
    ctx.create_agentsync_rule("test-rule", rule_content);

    let options = SyncOptions {
        dry_run: true,
        verbose: false,
    };
    let result = ctx.sync_to_tools(&options);

    assert_sync_result(&result, 1, 0, 0, 0);
    ctx.assert_rule_not_exists(Tool::Cursor, "test-rule");
}

#[test]
fn test_sync_to_tools_update_existing() {
    let ctx = TestContext::new().init_project_with_tools(&["cursor"]);

    let rule_content = r#"---
targets: ["*"]
description: "Test rule"
globs: "**/*"
---

# Test Rule

Version 1
"#;
    ctx.create_agentsync_rule("test-rule", rule_content);

    // First sync
    let result1 = ctx.sync_to_tools(&default_sync_options());
    assert_sync_result(&result1, 1, 0, 0, 0);

    // Second sync without changes
    let result2 = ctx.sync_to_tools(&default_sync_options());
    assert_sync_result(&result2, 0, 0, 1, 0);

    // Update the rule
    let updated_content = r#"---
targets: ["*"]
description: "Test rule"
globs: "**/*"
---

# Test Rule

Version 2 - Updated
"#;
    ctx.create_agentsync_rule("test-rule", updated_content);

    // Third sync with updated content
    let result3 = ctx.sync_to_tools(&default_sync_options());
    assert_sync_result(&result3, 0, 1, 0, 0);
}

#[test]
fn test_sync_from_cursor_to_agentsync() {
    let ctx = TestContext::new().init_project();

    let cursor_rule = r#"---
description: "Cursor rule"
alwaysApply: true
globs: ""
---

# Cursor Rule

This is from cursor.
"#;
    ctx.create_cursor_rule("cursor-rule", cursor_rule);

    let result = ctx.sync_from_tool(Tool::Cursor, &default_sync_options());

    assert_sync_result(&result, 1, 0, 0, 0);
    ctx.assert_rule_exists(Tool::AgentSync, "cursor-rule");

    let agentsync_content = ctx.read_rule(Tool::AgentSync, "cursor-rule");
    assert!(agentsync_content.contains("targets:"));
    assert!(agentsync_content.contains("description: Cursor rule"));
    assert!(agentsync_content.contains("This is from cursor."));
}

#[test]
fn test_sync_from_windsurf_to_agentsync() {
    let ctx = TestContext::new().init_project();

    let windsurf_rule = r#"---
trigger: glob
description: "Windsurf rule"
globs: "**/*.py"
---

# Windsurf Rule

This is from windsurf.
"#;
    ctx.create_windsurf_rule("windsurf-rule", windsurf_rule);

    let result = ctx.sync_from_tool(Tool::Windsurf, &default_sync_options());

    assert_sync_result(&result, 1, 0, 0, 0);
    ctx.assert_rule_exists(Tool::AgentSync, "windsurf-rule");

    let agentsync_content = ctx.read_rule(Tool::AgentSync, "windsurf-rule");
    assert!(agentsync_content.contains("targets:"));
    assert!(agentsync_content.contains("description: Windsurf rule"));
    assert!(agentsync_content.contains("**/*.py"));
}

#[test]
fn test_sync_from_copilot_to_agentsync() {
    let ctx = TestContext::new().init_project();

    let copilot_rule = r#"---
description: "Copilot rule"
applyTo: "**/*.js"
---

# Copilot Rule

This is from copilot.
"#;
    ctx.create_copilot_rule("copilot-rule", copilot_rule);

    let result = ctx.sync_from_tool(Tool::Copilot, &default_sync_options());

    assert_sync_result(&result, 1, 0, 0, 0);
    ctx.assert_rule_exists(Tool::AgentSync, "copilot-rule");

    let agentsync_content = ctx.read_rule(Tool::AgentSync, "copilot-rule");
    assert!(agentsync_content.contains("targets:"));
    assert!(agentsync_content.contains("description: Copilot rule"));
    assert!(agentsync_content.contains("**/*.js"));
}

#[test]
fn test_sync_from_tool_dry_run() {
    let ctx = TestContext::new().init_project();

    let cursor_rule = r#"---
description: "Test"
alwaysApply: false
globs: "**/*.rs"
---

# Test
"#;
    ctx.create_cursor_rule("test", cursor_rule);

    let options = SyncOptions {
        dry_run: true,
        verbose: false,
    };
    let result = ctx.sync_from_tool(Tool::Cursor, &options);

    assert_sync_result(&result, 1, 0, 0, 0);
    ctx.assert_rule_not_exists(Tool::AgentSync, "test");
}

#[test]
fn test_full_roundtrip_sync() {
    let ctx = TestContext::new().init_project();

    // Step 1: Create a Cursor rule
    let cursor_rule = r#"---
description: "Roundtrip test"
alwaysApply: false
globs: "**/*.ts"
---

# Roundtrip Test

Original content from Cursor.
"#;
    ctx.create_cursor_rule("roundtrip", cursor_rule);

    // Step 2: Import from Cursor to AgentSync
    let result1 = ctx.sync_from_tool(Tool::Cursor, &default_sync_options());
    assert_sync_result(&result1, 1, 0, 0, 0);

    // Step 3: Sync from AgentSync to all tools
    let result2 = ctx.sync_to_tools(&default_sync_options());

    // Should have some combination of added/updated/skipped
    assert_eq!(result2.total_processed(), 3); // All 3 tools
    assert_eq!(result2.errors.len(), 0); // No errors

    // Verify all files exist
    ctx.assert_rule_exists(Tool::Cursor, "roundtrip");
    ctx.assert_rule_exists(Tool::Copilot, "roundtrip");
    ctx.assert_rule_exists(Tool::Windsurf, "roundtrip");
}
