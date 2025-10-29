//! Data models for AgentSync and tool-specific rule formats
//!
//! This module defines the data structures for representing rules in different formats:
//! - AgentSync format (canonical, single source of truth)
//! - Cursor format (.mdc files)
//! - GitHub Copilot format (.md files)
//! - Windsurf format (.md files)

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// Windsurf trigger mode
///
/// Determines when a rule is applied:
/// - `Manual`: Activate via @mention
/// - `AlwaysOn`: Always Apply (always in context)
/// - `ModelDecision`: Apply Intelligently (model decides when relevant)
/// - `Glob`: Apply to Specific Files (based on file patterns)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumString, Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum WindsurfTrigger {
    Manual,
    AlwaysOn,
    ModelDecision,
    Glob,
}

impl Default for WindsurfTrigger {
    fn default() -> Self {
        Self::ModelDecision
    }
}

/// AgentSync rule format (canonical format)
///
/// This is the single source of truth format stored in `.agentsync/rules/*.md`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSyncRule {
    /// Target tools: `["cursor", "copilot", "windsurf"]` or `["*"]` for all
    #[serde(default = "default_targets")]
    pub targets: Vec<String>,

    #[serde(default)]
    pub description: String,

    /// Comma-separated glob patterns
    #[serde(default = "default_globs")]
    pub globs: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<CursorConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub windsurf: Option<WindsurfConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub copilot: Option<CopilotConfig>,
}

/// Cursor-specific rule configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CursorConfig {
    /// Always Apply (true) or Apply Intelligently/to Specific Files (false)
    #[serde(rename = "alwaysApply", default)]
    pub always_apply: bool,

    /// Comma-separated glob patterns
    #[serde(default)]
    pub globs: String,
}

/// Windsurf-specific rule configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindsurfConfig {
    #[serde(default)]
    pub trigger: WindsurfTrigger,

    /// Comma-separated glob patterns
    #[serde(default)]
    pub globs: String,
}

/// GitHub Copilot-specific rule configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CopilotConfig {
    /// Comma-separated glob patterns
    #[serde(rename = "applyTo", default = "default_copilot_apply_to")]
    pub apply_to: String,
}

/// Cursor rule format (.mdc files in .cursor/rules/)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CursorRule {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,

    #[serde(rename = "alwaysApply", default)]
    pub always_apply: bool,

    /// Comma-separated glob patterns
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub globs: String,
}

/// Windsurf rule format (.md files in .windsurf/rules/)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindsurfRule {
    #[serde(default)]
    pub trigger: WindsurfTrigger,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,

    /// Comma-separated glob patterns
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub globs: String,
}

/// GitHub Copilot rule format (.md files in .github/instructions/)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CopilotRule {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,

    /// Comma-separated glob patterns
    #[serde(rename = "applyTo", default = "default_copilot_apply_to")]
    pub apply_to: String,
}

/// AgentSync configuration (agentsync.json)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSyncConfig {
    #[serde(default = "default_tools")]
    pub tools: Vec<String>,

    /// Base directories for monorepo support
    #[serde(rename = "baseDirs", default = "default_base_dirs")]
    pub base_dirs: Vec<String>,
}

/// Rule with its content (frontmatter + body)
#[derive(Debug, Clone, PartialEq)]
pub struct Rule<T> {
    pub frontmatter: T,

    /// Markdown body after frontmatter
    pub content: String,
}

// Default value functions

fn default_targets() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_globs() -> String {
    "**/*".to_string()
}

fn default_copilot_apply_to() -> String {
    "**".to_string()
}

fn default_tools() -> Vec<String> {
    vec![
        "cursor".to_string(),
        "copilot".to_string(),
        "windsurf".to_string(),
    ]
}

fn default_base_dirs() -> Vec<String> {
    vec![".".to_string()]
}

impl AgentSyncConfig {
    /// Validate the configuration
    ///
    /// Performs comprehensive validation including:
    /// - Tool names are valid
    /// - baseDirs is not empty
    /// - baseDirs paths don't contain path traversal sequences
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any tool name is invalid (provides suggestions)
    /// - baseDirs is empty
    /// - Any baseDir contains path traversal sequences
    pub fn validate(&self) -> crate::Result<()> {
        // Validate tools with helpful error messages
        const VALID_TOOLS: &[&str] = &["cursor", "copilot", "windsurf"];

        for tool in &self.tools {
            if !VALID_TOOLS.contains(&tool.as_str()) {
                // Provide suggestions for typos
                let suggestion = match tool.to_lowercase().as_str() {
                    "github-copilot" | "github_copilot" | "githubcopilot"
                    | "vscode-copilot" | "vscode_copilot" =>
                        Some("Did you mean 'copilot'?"),
                    "cascade" | "codeium" =>
                        Some("This tool is not yet supported"),
                    _ => None,
                };

                let mut error_msg = format!(
                    "Invalid tool name: '{}'\n\nValid tools: {}",
                    tool,
                    VALID_TOOLS.join(", ")
                );

                if let Some(hint) = suggestion {
                    error_msg.push_str("\n\n");
                    error_msg.push_str(hint);
                }

                return Err(crate::AgentSyncError::ConfigError {
                    error: error_msg,
                });
            }
        }

        // Validate base_dirs using security module
        crate::security::validate_base_dirs(&self.base_dirs)?;

        Ok(())
    }
}

impl Default for AgentSyncConfig {
    fn default() -> Self {
        Self {
            tools: default_tools(),
            base_dirs: default_base_dirs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agentsync_config_validation() {
        let config = AgentSyncConfig::default();
        assert!(config.validate().is_ok());

        let invalid_config = AgentSyncConfig {
            tools: vec!["invalid".to_string()],
            base_dirs: vec![".".to_string()],
        };
        assert!(invalid_config.validate().is_err());

        let empty_dirs_config = AgentSyncConfig {
            tools: vec!["cursor".to_string()],
            base_dirs: vec![],
        };
        assert!(empty_dirs_config.validate().is_err());
    }
}
