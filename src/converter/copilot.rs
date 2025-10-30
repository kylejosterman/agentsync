//! Copilot-specific conversions

use super::{
    ConfigMode, GLOB_UNIVERSAL_DOUBLE_STAR, TARGET_ALL, create_all_configs, is_universal_glob,
    normalize_globs,
};
use crate::models::{AgentSyncRule, CopilotRule, Rule};

/// Convert Copilot rule to `AgentSync` rule
#[must_use]
pub fn copilot_to_agentsync(copilot_rule: &CopilotRule) -> AgentSyncRule {
    let mode = if is_universal_glob(&copilot_rule.apply_to) {
        ConfigMode::AlwaysOn
    } else {
        ConfigMode::Glob(&copilot_rule.apply_to)
    };

    let (cursor_config, windsurf_config, copilot_config, globs) = create_all_configs(&mode);

    AgentSyncRule {
        targets: vec![TARGET_ALL.to_string()],
        description: copilot_rule.description.clone(),
        globs,
        cursor: Some(cursor_config),
        windsurf: Some(windsurf_config),
        copilot: Some(copilot_config),
    }
}

/// Convert `AgentSync` rule to Copilot rule
#[must_use]
pub fn agentsync_to_copilot(agentsync_rule: &AgentSyncRule) -> CopilotRule {
    let copilot_config = agentsync_rule.copilot.as_ref();

    CopilotRule {
        description: agentsync_rule.description.clone(),
        apply_to: copilot_config.map_or_else(
            || {
                // Use global globs if no copilot-specific config
                if is_universal_glob(&agentsync_rule.globs) {
                    GLOB_UNIVERSAL_DOUBLE_STAR.to_string()
                } else {
                    normalize_globs(&agentsync_rule.globs)
                }
            },
            |c| normalize_globs(&c.apply_to),
        ),
    }
}

/// Convert Copilot rule with content to `AgentSync` rule
#[must_use]
pub fn copilot_rule_to_agentsync(rule: &Rule<CopilotRule>) -> Rule<AgentSyncRule> {
    Rule {
        frontmatter: copilot_to_agentsync(&rule.frontmatter),
        content: rule.content.clone(),
    }
}

/// Convert `AgentSync` rule with content to Copilot rule
#[must_use]
pub fn agentsync_rule_to_copilot(rule: &Rule<AgentSyncRule>) -> Rule<CopilotRule> {
    Rule {
        frontmatter: agentsync_to_copilot(&rule.frontmatter),
        content: rule.content.clone(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::models::{CopilotConfig, WindsurfTrigger};

    #[test]
    fn test_copilot_to_agentsync_universal() {
        let copilot = CopilotRule {
            description: "Test rule".to_string(),
            apply_to: "**".to_string(),
        };

        let agentsync = copilot_to_agentsync(&copilot);
        assert_eq!(agentsync.globs, "**/*");

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
    fn test_copilot_to_agentsync_specific_pattern() {
        let copilot = CopilotRule {
            description: "Python rule".to_string(),
            apply_to: "**/*.py".to_string(),
        };

        let agentsync = copilot_to_agentsync(&copilot);
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
    fn test_agentsync_to_copilot() {
        let agentsync = AgentSyncRule {
            targets: vec!["*".to_string()],
            description: "Test rule".to_string(),
            globs: "**/*.rs".to_string(),
            cursor: None,
            windsurf: None,
            copilot: Some(CopilotConfig {
                apply_to: "**/*.rs".to_string(),
            }),
        };

        let copilot = agentsync_to_copilot(&agentsync);

        assert_eq!(copilot.description, "Test rule");
        assert_eq!(copilot.apply_to, "**/*.rs");
    }

    #[test]
    fn test_agentsync_to_copilot_fallback() {
        let agentsync = AgentSyncRule {
            targets: vec!["*".to_string()],
            description: "Test rule".to_string(),
            globs: "**/*.rs".to_string(),
            cursor: None,
            windsurf: None,
            copilot: None,
        };

        let copilot = agentsync_to_copilot(&agentsync);

        assert_eq!(copilot.description, "Test rule");
        assert_eq!(copilot.apply_to, "**/*.rs");
    }

    #[test]
    fn test_roundtrip_copilot_to_agentsync_to_copilot() {
        let original = CopilotRule {
            description: "Roundtrip test".to_string(),
            apply_to: "**/*.py".to_string(),
        };

        let agentsync = copilot_to_agentsync(&original);
        let back_to_copilot = agentsync_to_copilot(&agentsync);

        assert_eq!(original.description, back_to_copilot.description);
        assert_eq!(original.apply_to, back_to_copilot.apply_to);
    }
}
