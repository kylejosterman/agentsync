//! Common test helpers and utilities
//!
//! This module provides reusable test infrastructure

// The `unreachable_pub` is to silence false positives in IDEs.
// The `dead_code` is because not all test utilities are used by all tests.
#![allow(dead_code, unreachable_pub)]

use agentsync::fs::Tool;
use agentsync::sync::{SyncOptions, SyncResult};
use fs_err as fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test context that manages a temporary project directory
///
/// Automatically cleans up on drop. Provides convenient methods for:
/// - Project initialization
/// - Rule creation
/// - Fixture loading
/// - File assertions
pub struct TestContext {
    /// Temporary directory (auto-cleaned on drop)
    temp_dir: TempDir,
}

impl TestContext {
    /// Create a new test context with a temporary directory
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().expect("Failed to create temp dir"),
        }
    }

    /// Get the project root path
    pub fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Get a path relative to the project root
    pub fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root().join(relative)
    }

    /// Initialize a project with .agentsync/rules/ and agentsync.json
    pub fn init_project(self) -> Self {
        self.init_project_with_tools(&["cursor", "copilot", "windsurf"])
    }

    /// Initialize project with specific tools enabled
    pub fn init_project_with_tools(self, tools: &[&str]) -> Self {
        // Create .agentsync/rules/
        let agentsync_dir = self.path(".agentsync/rules");
        fs::create_dir_all(&agentsync_dir).expect("Failed to create .agentsync/rules");

        // Create agentsync.json
        let config = format!(
            r#"{{
  "tools": [{}],
  "baseDirs": ["."]
}}"#,
            tools
                .iter()
                .map(|t| format!("\"{t}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
        fs::write(self.path("agentsync.json"), config).expect("Failed to write agentsync.json");

        self
    }

    /// Create an AgentSync rule file
    pub fn create_agentsync_rule(&self, name: &str, content: &str) -> PathBuf {
        let path = self.path(format!(".agentsync/rules/{name}.md"));
        fs::create_dir_all(path.parent().unwrap()).expect("Failed to create parent dir");
        fs::write(&path, content).expect("Failed to write rule");
        path
    }

    /// Create a Cursor rule file
    pub fn create_cursor_rule(&self, name: &str, content: &str) -> PathBuf {
        let path = self.path(format!(".cursor/rules/{name}.mdc"));
        fs::create_dir_all(path.parent().unwrap()).expect("Failed to create parent dir");
        fs::write(&path, content).expect("Failed to write rule");
        path
    }

    /// Create a Copilot rule file
    pub fn create_copilot_rule(&self, name: &str, content: &str) -> PathBuf {
        let path = self.path(format!(".github/instructions/{name}.instructions.md"));
        fs::create_dir_all(path.parent().unwrap()).expect("Failed to create parent dir");
        fs::write(&path, content).expect("Failed to write rule");
        path
    }

    /// Create a Windsurf rule file
    pub fn create_windsurf_rule(&self, name: &str, content: &str) -> PathBuf {
        let path = self.path(format!(".windsurf/rules/{name}.md"));
        fs::create_dir_all(path.parent().unwrap()).expect("Failed to create parent dir");
        fs::write(&path, content).expect("Failed to write rule");
        path
    }

    /// Create a rule for a specific tool
    pub fn create_rule(&self, tool: Tool, name: &str, content: &str) -> PathBuf {
        match tool {
            Tool::AgentSync => self.create_agentsync_rule(name, content),
            Tool::Cursor => self.create_cursor_rule(name, content),
            Tool::Copilot => self.create_copilot_rule(name, content),
            Tool::Windsurf => self.create_windsurf_rule(name, content),
        }
    }

    /// Copy fixture files from tests/fixtures/{tool}/ to the project
    pub fn copy_fixtures(&self, tool: Tool) -> Vec<PathBuf> {
        let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(tool.directory().split('/').next_back().unwrap());

        if !fixture_dir.exists() {
            return Vec::new();
        }

        let target_dir = self.path(tool.directory());
        fs::create_dir_all(&target_dir).expect("Failed to create target dir");

        let mut copied = Vec::new();
        for entry in fs::read_dir(&fixture_dir).expect("Failed to read fixture dir") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy();

            // For Copilot, match .instructions.md files; for others, match by extension
            let should_copy = match tool {
                Tool::Copilot => filename_str.ends_with(".instructions.md"),
                _ => path.extension().and_then(|s| s.to_str()) == Some(tool.extension()),
            };

            if should_copy {
                let dest = target_dir.join(entry.file_name());
                fs::copy(&path, &dest).expect("Failed to copy fixture");
                copied.push(dest);
            }
        }

        copied
    }

    /// Assert that a rule file exists for a tool
    pub fn assert_rule_exists(&self, tool: Tool, name: &str) {
        let path = agentsync::fs::rule_path(self.root(), tool, name)
            .expect("Failed to construct rule path");
        assert!(
            path.exists(),
            "Expected rule '{name}' to exist at {}",
            path.display()
        );
    }

    /// Assert that a rule file does NOT exist for a tool
    pub fn assert_rule_not_exists(&self, tool: Tool, name: &str) {
        let path = agentsync::fs::rule_path(self.root(), tool, name)
            .expect("Failed to construct rule path");
        assert!(
            !path.exists(),
            "Expected rule '{name}' to NOT exist at {}",
            path.display()
        );
    }

    /// Read a rule file content
    pub fn read_rule(&self, tool: Tool, name: &str) -> String {
        let path = agentsync::fs::rule_path(self.root(), tool, name)
            .expect("Failed to construct rule path");
        fs::read_to_string(&path).expect("Failed to read rule")
    }

    /// Load the agentsync.json config
    pub fn load_config(&self) -> agentsync::models::AgentSyncConfig {
        agentsync::config::load_config(self.path("agentsync.json")).expect("Failed to load config")
    }

    /// Run sync to tools
    pub fn sync_to_tools(&self, options: &SyncOptions) -> SyncResult {
        let config = self.load_config();
        agentsync::sync::sync_to_tools(self.root(), &config.tools, options)
            .expect("Sync to tools failed")
    }

    /// Run sync from a specific tool
    pub fn sync_from_tool(&self, tool: Tool, options: &SyncOptions) -> SyncResult {
        agentsync::sync::sync_from_tool(self.root(), tool, options).expect("Sync from tool failed")
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Create default sync options for tests
pub fn default_sync_options() -> SyncOptions {
    SyncOptions {
        dry_run: false,
        verbose: false,
    }
}

/// Create a simple AgentSync rule template
pub fn simple_agentsync_rule(description: &str, globs: &str) -> String {
    format!(
        r#"---
targets:
  - "*"
description: "{description}"
globs: "{globs}"
cursor:
  alwaysApply: false
  globs: "{globs}"
windsurf:
  trigger: glob
  globs: "{globs}"
copilot:
  applyTo: "{globs}"
---

# Test Rule

This is a test rule.
"#
    )
}

/// Create a simple Cursor rule template
pub fn simple_cursor_rule(description: &str, always_apply: bool, globs: &str) -> String {
    format!(
        r#"---
description: "{description}"
alwaysApply: {always_apply}
globs: "{globs}"
---

# Cursor Rule

This is a cursor rule.
"#
    )
}

/// Assert sync result has expected counts
pub fn assert_sync_result(
    result: &SyncResult,
    added: usize,
    updated: usize,
    skipped: usize,
    errors: usize,
) {
    assert_eq!(
        result.added.len(),
        added,
        "Expected {added} added, got {}",
        result.added.len()
    );
    assert_eq!(
        result.updated.len(),
        updated,
        "Expected {updated} updated, got {}",
        result.updated.len()
    );
    assert_eq!(
        result.skipped.len(),
        skipped,
        "Expected {skipped} skipped, got {}",
        result.skipped.len()
    );
    assert_eq!(
        result.errors.len(),
        errors,
        "Expected {errors} errors, got {}",
        result.errors.len()
    );
}
