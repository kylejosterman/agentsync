//! Cursor-specific conversions

use super::{ConfigMode, TARGET_ALL, create_all_configs, is_universal_glob, normalize_globs};
use crate::models::{AgentSyncRule, CursorRule, Rule};

/// Convert Cursor rule to `AgentSync` format with inference
#[must_use]
pub fn cursor_to_agentsync(cursor_rule: &CursorRule) -> AgentSyncRule {
    let mode = if cursor_rule.always_apply {
        ConfigMode::AlwaysOn
    } else if !cursor_rule.globs.is_empty() {
        ConfigMode::Glob(&cursor_rule.globs)
    } else if !cursor_rule.description.is_empty() {
        ConfigMode::Intelligent
    } else {
        ConfigMode::Manual
    };

    let (cursor_config, windsurf_config, copilot_config, globs) = create_all_configs(&mode);

    AgentSyncRule {
        targets: vec![TARGET_ALL.to_string()],
        description: cursor_rule.description.clone(),
        globs,
        cursor: Some(cursor_config),
        windsurf: Some(windsurf_config),
        copilot: Some(copilot_config),
    }
}

/// Convert `AgentSync` rule to Cursor format
#[must_use]
pub fn agentsync_to_cursor(agentsync_rule: &AgentSyncRule) -> CursorRule {
    let cursor_config = agentsync_rule.cursor.as_ref();
    let always_apply = cursor_config.is_some_and(|c| c.always_apply);

    // For Always Apply mode, Cursor should not have description or globs in frontmatter
    let (description, globs) = if always_apply {
        (String::new(), String::new())
    } else {
        let globs = cursor_config.map_or_else(
            || {
                // Fallback: use global globs if no cursor-specific config
                if is_universal_glob(&agentsync_rule.globs) {
                    String::new()
                } else {
                    normalize_globs(&agentsync_rule.globs)
                }
            },
            |c| normalize_globs(&c.globs),
        );
        (agentsync_rule.description.clone(), globs)
    };

    CursorRule {
        description,
        always_apply,
        globs,
    }
}

/// Convert Cursor rule with content to `AgentSync` format
#[must_use]
pub fn cursor_rule_to_agentsync(rule: &Rule<CursorRule>) -> Rule<AgentSyncRule> {
    Rule {
        frontmatter: cursor_to_agentsync(&rule.frontmatter),
        content: rule.content.clone(),
    }
}

/// Convert `AgentSync` rule with content to Cursor format
#[must_use]
pub fn agentsync_rule_to_cursor(rule: &Rule<AgentSyncRule>) -> Rule<CursorRule> {
    Rule {
        frontmatter: agentsync_to_cursor(&rule.frontmatter),
        content: rule.content.clone(),
    }
}

#[cfg(test)]
mod tests {
    // Allow expect/unwrap in tests for brevity
    #![allow(clippy::expect_used)]
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::models::{CursorConfig, WindsurfTrigger};

    #[test]
    fn test_cursor_to_agentsync_always_mode() {
        let cursor = CursorRule {
            description: "Test rule".to_string(),
            always_apply: true,
            globs: String::new(),
        };

        let agentsync = cursor_to_agentsync(&cursor);

        assert_eq!(agentsync.description, "Test rule");
        assert_eq!(agentsync.globs, "**/*");
        assert_eq!(agentsync.targets, vec!["*"]);

        let cursor_cfg = agentsync.cursor.expect("should have cursor config");
        assert!(cursor_cfg.always_apply);
        assert_eq!(cursor_cfg.globs, "");

        let windsurf_cfg = agentsync.windsurf.expect("should have windsurf config");
        assert_eq!(windsurf_cfg.trigger, WindsurfTrigger::AlwaysOn);
        assert_eq!(windsurf_cfg.globs, "");

        let copilot_cfg = agentsync.copilot.expect("should have copilot config");
        assert_eq!(copilot_cfg.apply_to, "**");
    }

    #[test]
    fn test_cursor_to_agentsync_auto_attached_with_globs() {
        let cursor = CursorRule {
            description: "Python rule".to_string(),
            always_apply: false,
            globs: "**/*.py".to_string(),
        };

        let agentsync = cursor_to_agentsync(&cursor);

        assert_eq!(agentsync.globs, "**/*.py");

        let cursor_cfg = agentsync.cursor.expect("should have cursor config");
        assert!(!cursor_cfg.always_apply);
        assert_eq!(cursor_cfg.globs, "**/*.py");

        let windsurf_cfg = agentsync.windsurf.expect("should have windsurf config");
        assert_eq!(windsurf_cfg.trigger, WindsurfTrigger::Glob);
        assert_eq!(windsurf_cfg.globs, "**/*.py");

        let copilot_cfg = agentsync.copilot.expect("should have copilot config");
        assert_eq!(copilot_cfg.apply_to, "**/*.py");
    }

    #[test]
    fn test_cursor_to_agentsync_manual_mode() {
        let cursor = CursorRule {
            description: String::new(), // No description for true manual mode
            always_apply: false,
            globs: String::new(),
        };

        let agentsync = cursor_to_agentsync(&cursor);

        assert_eq!(agentsync.globs, "**/*");

        let windsurf_cfg = agentsync.windsurf.expect("should have windsurf config");
        assert_eq!(windsurf_cfg.trigger, WindsurfTrigger::Manual);
    }

    #[test]
    fn test_agentsync_to_cursor() {
        let agentsync = AgentSyncRule {
            targets: vec!["*".to_string()],
            description: "Test rule".to_string(),
            globs: "**/*.rs".to_string(),
            cursor: Some(CursorConfig {
                always_apply: false,
                globs: "**/*.rs".to_string(),
            }),
            windsurf: None,
            copilot: None,
        };

        let cursor = agentsync_to_cursor(&agentsync);

        assert_eq!(cursor.description, "Test rule");
        assert!(!cursor.always_apply);
        assert_eq!(cursor.globs, "**/*.rs");
    }

    #[test]
    fn test_agentsync_to_cursor_fallback() {
        let agentsync = AgentSyncRule {
            targets: vec!["*".to_string()],
            description: "Test rule".to_string(),
            globs: "**/*.rs".to_string(),
            cursor: None,
            windsurf: None,
            copilot: None,
        };

        let cursor = agentsync_to_cursor(&agentsync);

        assert_eq!(cursor.description, "Test rule");
        assert!(!cursor.always_apply);
        assert_eq!(cursor.globs, "**/*.rs");
    }

    #[test]
    fn test_cursor_rule_to_agentsync_with_content() {
        let cursor_rule = Rule {
            frontmatter: CursorRule {
                description: "Test rule".to_string(),
                always_apply: true,
                globs: String::new(),
            },
            content: "# Test Content\n\nRule body here.".to_string(),
        };

        let agentsync_rule = cursor_rule_to_agentsync(&cursor_rule);

        assert_eq!(agentsync_rule.frontmatter.description, "Test rule");
        assert_eq!(agentsync_rule.content, "# Test Content\n\nRule body here.");
    }

    #[test]
    fn test_agentsync_rule_to_cursor_with_content() {
        let agentsync_rule = Rule {
            frontmatter: AgentSyncRule {
                targets: vec!["*".to_string()],
                description: "Test rule".to_string(),
                globs: "**/*.rs".to_string(),
                cursor: Some(CursorConfig {
                    always_apply: false,
                    globs: "**/*.rs".to_string(),
                }),
                windsurf: None,
                copilot: None,
            },
            content: "# Test Content\n\nRule body here.".to_string(),
        };

        let cursor_rule = agentsync_rule_to_cursor(&agentsync_rule);

        assert_eq!(cursor_rule.frontmatter.description, "Test rule");
        assert_eq!(cursor_rule.content, "# Test Content\n\nRule body here.");
    }

    #[test]
    fn test_roundtrip_cursor_to_agentsync_to_cursor() {
        let original = CursorRule {
            description: "Roundtrip test".to_string(),
            always_apply: false,
            globs: "**/*.py".to_string(),
        };

        let agentsync = cursor_to_agentsync(&original);
        let back_to_cursor = agentsync_to_cursor(&agentsync);

        assert_eq!(original.description, back_to_cursor.description);
        assert_eq!(original.always_apply, back_to_cursor.always_apply);
        assert_eq!(original.globs, back_to_cursor.globs);
    }
}
