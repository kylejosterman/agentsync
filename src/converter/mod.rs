//! Bidirectional conversion between AgentSync and tool formats.

use crate::models::{CopilotConfig, CursorConfig, WindsurfConfig, WindsurfTrigger};
use itertools::Itertools;

mod copilot;
mod cursor;
mod windsurf;

// Re-export conversion functions
pub use copilot::{
    agentsync_rule_to_copilot, agentsync_to_copilot, copilot_rule_to_agentsync,
    copilot_to_agentsync,
};
pub use cursor::{
    agentsync_rule_to_cursor, agentsync_to_cursor, cursor_rule_to_agentsync, cursor_to_agentsync,
};
pub use windsurf::{
    agentsync_rule_to_windsurf, agentsync_to_windsurf, windsurf_rule_to_agentsync,
    windsurf_to_agentsync,
};

// Common constants
pub(crate) const GLOB_UNIVERSAL_DOUBLE_STAR: &str = "**";
pub(crate) const GLOB_UNIVERSAL_RECURSIVE: &str = "**/*";
pub(crate) const TARGET_ALL: &str = "*";

/// Normalize globs by trimming whitespace around commas
#[must_use]
pub fn normalize_globs(globs: &str) -> String {
    if globs.is_empty() {
        return String::new();
    }
    globs.split(',').map(str::trim).format(",").to_string()
}

/// Check if glob is universal (applies to all files)
pub(crate) fn is_universal_glob(globs: &str) -> bool {
    let normalized = globs.trim();
    normalized.is_empty()
        || normalized == GLOB_UNIVERSAL_RECURSIVE
        || normalized == GLOB_UNIVERSAL_DOUBLE_STAR
}

/// Unified configuration mode
#[derive(Debug, Clone)]
pub(crate) enum ConfigMode<'a> {
    AlwaysOn,
    Manual,
    Intelligent,
    Glob(&'a str),
}

/// Create tool configs from unified mode
pub(crate) fn create_all_configs(
    mode: &ConfigMode<'_>,
) -> (CursorConfig, WindsurfConfig, CopilotConfig, String) {
    match mode {
        ConfigMode::AlwaysOn => (
            CursorConfig {
                always_apply: true,
                globs: String::new(),
            },
            WindsurfConfig {
                trigger: WindsurfTrigger::AlwaysOn,
                globs: String::new(),
            },
            CopilotConfig {
                apply_to: GLOB_UNIVERSAL_DOUBLE_STAR.to_string(),
            },
            GLOB_UNIVERSAL_RECURSIVE.to_string(),
        ),
        ConfigMode::Manual => (
            CursorConfig {
                always_apply: false,
                globs: String::new(),
            },
            WindsurfConfig {
                trigger: WindsurfTrigger::Manual,
                globs: String::new(),
            },
            CopilotConfig {
                apply_to: GLOB_UNIVERSAL_DOUBLE_STAR.to_string(),
            },
            GLOB_UNIVERSAL_RECURSIVE.to_string(),
        ),
        ConfigMode::Intelligent => (
            CursorConfig {
                always_apply: false,
                globs: String::new(),
            },
            WindsurfConfig {
                trigger: WindsurfTrigger::ModelDecision,
                globs: String::new(),
            },
            CopilotConfig {
                apply_to: GLOB_UNIVERSAL_DOUBLE_STAR.to_string(),
            },
            GLOB_UNIVERSAL_RECURSIVE.to_string(),
        ),
        ConfigMode::Glob(globs) => {
            let normalized = normalize_globs(globs);
            (
                CursorConfig {
                    always_apply: false,
                    globs: normalized.clone(),
                },
                WindsurfConfig {
                    trigger: WindsurfTrigger::Glob,
                    globs: normalized.clone(),
                },
                CopilotConfig {
                    apply_to: normalized.clone(),
                },
                normalized,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_globs_empty() {
        assert_eq!(normalize_globs(""), "");
    }

    #[test]
    fn test_normalize_globs_single() {
        assert_eq!(normalize_globs("**/*.rs"), "**/*.rs");
    }

    #[test]
    fn test_normalize_globs_multiple() {
        assert_eq!(
            normalize_globs("src/**/*.py,tests/**/*.py"),
            "src/**/*.py,tests/**/*.py"
        );
    }

    #[test]
    fn test_normalize_globs_with_spaces() {
        assert_eq!(
            normalize_globs("  src/**/*.py  ,  tests/**/*.py  "),
            "src/**/*.py,tests/**/*.py"
        );
    }

    #[test]
    fn test_is_universal_glob() {
        assert!(is_universal_glob(""));
        assert!(is_universal_glob("**/*"));
        assert!(is_universal_glob("**"));
        assert!(is_universal_glob("  **/*  "));
        assert!(!is_universal_glob("**/*.py"));
        assert!(!is_universal_glob("src/**/*"));
    }
}
