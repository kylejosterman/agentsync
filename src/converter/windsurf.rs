//! Windsurf-specific conversions

use super::{ConfigMode, TARGET_ALL, create_all_configs, is_universal_glob, normalize_globs};
use crate::models::{AgentSyncRule, Rule, WindsurfRule, WindsurfTrigger};

/// Convert Windsurf rule to `AgentSync` format
#[must_use]
pub fn windsurf_to_agentsync(windsurf_rule: &WindsurfRule) -> AgentSyncRule {
    let mode = match windsurf_rule.trigger {
        WindsurfTrigger::AlwaysOn => ConfigMode::AlwaysOn,
        WindsurfTrigger::Glob => ConfigMode::Glob(&windsurf_rule.globs),
        WindsurfTrigger::ModelDecision => ConfigMode::Intelligent,
        WindsurfTrigger::Manual => ConfigMode::Manual,
    };

    let (cursor_config, mut windsurf_config, copilot_config, globs) = create_all_configs(&mode);

    // Preserve the original Windsurf trigger mode
    windsurf_config.trigger = windsurf_rule.trigger.clone();
    windsurf_config.globs = normalize_globs(&windsurf_rule.globs);

    AgentSyncRule {
        targets: vec![TARGET_ALL.to_string()],
        description: windsurf_rule.description.clone(),
        globs,
        cursor: Some(cursor_config),
        windsurf: Some(windsurf_config),
        copilot: Some(copilot_config),
    }
}

/// Convert `AgentSync` rule to Windsurf format
#[must_use]
pub fn agentsync_to_windsurf(agentsync_rule: &AgentSyncRule) -> WindsurfRule {
    let windsurf_config = agentsync_rule.windsurf.as_ref();
    let trigger = windsurf_config.map_or_else(Default::default, |c| c.trigger.clone());

    // Always apply should not have description or globs
    let (description, globs) = if trigger == WindsurfTrigger::AlwaysOn {
        (String::new(), String::new())
    } else {
        let globs = windsurf_config.map_or_else(
            || {
                // Use global globs if no windsurf-specific config
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

    WindsurfRule {
        trigger,
        description,
        globs,
    }
}

/// Convert Windsurf rule with content to `AgentSync` rule
#[must_use]
pub fn windsurf_rule_to_agentsync(rule: &Rule<WindsurfRule>) -> Rule<AgentSyncRule> {
    Rule {
        frontmatter: windsurf_to_agentsync(&rule.frontmatter),
        content: rule.content.clone(),
    }
}

/// Convert `AgentSync` rule with content to Windsurf rule
#[must_use]
pub fn agentsync_rule_to_windsurf(rule: &Rule<AgentSyncRule>) -> Rule<WindsurfRule> {
    Rule {
        frontmatter: agentsync_to_windsurf(&rule.frontmatter),
        content: rule.content.clone(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::models::WindsurfConfig;

    #[test]
    fn test_windsurf_to_agentsync_always_on() {
        let windsurf = WindsurfRule {
            trigger: WindsurfTrigger::AlwaysOn,
            description: "Test rule".to_string(),
            globs: String::new(),
        };

        let agentsync = windsurf_to_agentsync(&windsurf);

        assert_eq!(agentsync.globs, "**/*");

        let cursor_cfg = agentsync.cursor.expect("should have cursor config");
        assert!(cursor_cfg.always_apply);
        assert_eq!(cursor_cfg.globs, "");

        let windsurf_cfg = agentsync.windsurf.expect("should have windsurf config");
        assert_eq!(windsurf_cfg.trigger, WindsurfTrigger::AlwaysOn);

        let copilot_cfg = agentsync.copilot.expect("should have copilot config");
        assert_eq!(copilot_cfg.apply_to, "**");
    }

    #[test]
    fn test_windsurf_to_agentsync_glob_mode() {
        let windsurf = WindsurfRule {
            trigger: WindsurfTrigger::Glob,
            description: "Python rule".to_string(),
            globs: "src/**/*.py, tests/**/*.py".to_string(),
        };

        let agentsync = windsurf_to_agentsync(&windsurf);
        assert_eq!(agentsync.globs, "src/**/*.py,tests/**/*.py");

        let cursor_cfg = agentsync.cursor.expect("should have cursor config");
        assert!(!cursor_cfg.always_apply);
        assert_eq!(cursor_cfg.globs, "src/**/*.py,tests/**/*.py");

        let windsurf_cfg = agentsync.windsurf.expect("should have windsurf config");
        assert_eq!(windsurf_cfg.trigger, WindsurfTrigger::Glob);
        assert_eq!(windsurf_cfg.globs, "src/**/*.py,tests/**/*.py");

        let copilot_cfg = agentsync.copilot.expect("should have copilot config");
        assert_eq!(copilot_cfg.apply_to, "src/**/*.py,tests/**/*.py");
    }

    #[test]
    fn test_windsurf_to_agentsync_model_decision() {
        let windsurf = WindsurfRule {
            trigger: WindsurfTrigger::ModelDecision,
            description: "Smart rule".to_string(),
            globs: String::new(),
        };

        let agentsync = windsurf_to_agentsync(&windsurf);

        assert_eq!(agentsync.globs, "**/*");

        let cursor_cfg = agentsync.cursor.expect("should have cursor config");
        assert!(!cursor_cfg.always_apply);
        assert_eq!(cursor_cfg.globs, "");
    }

    #[test]
    fn test_agentsync_to_windsurf() {
        let agentsync = AgentSyncRule {
            targets: vec!["*".to_string()],
            description: "Test rule".to_string(),
            globs: "**/*.rs".to_string(),
            cursor: None,
            windsurf: Some(WindsurfConfig {
                trigger: WindsurfTrigger::Glob,
                globs: "**/*.rs".to_string(),
            }),
            copilot: None,
        };

        let windsurf = agentsync_to_windsurf(&agentsync);

        assert_eq!(windsurf.description, "Test rule");
        assert_eq!(windsurf.trigger, WindsurfTrigger::Glob);
        assert_eq!(windsurf.globs, "**/*.rs");
    }

    #[test]
    fn test_agentsync_to_windsurf_fallback() {
        let agentsync = AgentSyncRule {
            targets: vec!["*".to_string()],
            description: "Test rule".to_string(),
            globs: "**/*.rs".to_string(),
            cursor: None,
            windsurf: None,
            copilot: None,
        };

        let windsurf = agentsync_to_windsurf(&agentsync);

        assert_eq!(windsurf.description, "Test rule");
        assert_eq!(windsurf.trigger, WindsurfTrigger::ModelDecision);
        assert_eq!(windsurf.globs, "**/*.rs");
    }

    #[test]
    fn test_roundtrip_windsurf_to_agentsync_to_windsurf() {
        let original = WindsurfRule {
            trigger: WindsurfTrigger::Glob,
            description: "Roundtrip test".to_string(),
            globs: "src/**/*.py,tests/**/*.py".to_string(),
        };

        let agentsync = windsurf_to_agentsync(&original);
        let back_to_windsurf = agentsync_to_windsurf(&agentsync);

        assert_eq!(original.description, back_to_windsurf.description);
        assert_eq!(original.trigger, back_to_windsurf.trigger);
        assert_eq!(original.globs, back_to_windsurf.globs);
    }
}
