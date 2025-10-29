//! Windsurf tool processor implementation

use super::Processor;
use crate::converter::{agentsync_rule_to_windsurf, windsurf_rule_to_agentsync};
use crate::fs::Tool;
use crate::models::{AgentSyncRule, Rule, WindsurfRule};
use crate::parser::{parse_frontmatter, serialize_frontmatter};
use crate::Result;

/// Processor for Windsurf (.md files in .windsurf/rules/)
pub struct WindsurfProcessor;

impl Processor for WindsurfProcessor {
    fn tool(&self) -> Tool {
        Tool::Windsurf
    }

    fn convert_from_agentsync(&self, rule: &Rule<AgentSyncRule>) -> Result<String> {
        let windsurf_rule = agentsync_rule_to_windsurf(rule);
        serialize_frontmatter(&windsurf_rule)
    }

    fn convert_to_agentsync(&self, content: &str, path: &str) -> Result<Rule<AgentSyncRule>> {
        let windsurf_rule: Rule<WindsurfRule> = parse_frontmatter(content, Some(path))?;
        Ok(windsurf_rule_to_agentsync(&windsurf_rule))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{WindsurfConfig, WindsurfTrigger};

    #[test]
    fn test_windsurf_processor_tool() {
        let processor = WindsurfProcessor;
        assert_eq!(processor.tool(), Tool::Windsurf);
    }

    #[test]
    fn test_windsurf_processor_convert_from_agentsync() {
        let processor = WindsurfProcessor;

        let agentsync_rule = Rule {
            frontmatter: AgentSyncRule {
                targets: vec!["windsurf".to_string()],
                description: "Test rule".to_string(),
                globs: "**/*.rs".to_string(),
                cursor: None,
                windsurf: Some(WindsurfConfig {
                    trigger: WindsurfTrigger::Glob, // Use Glob to test description in frontmatter
                    globs: "**/*.rs".to_string(),
                }),
                copilot: None,
            },
            content: "# Test Rule\n\nThis is a test.".to_string(),
        };

        let result = processor.convert_from_agentsync(&agentsync_rule);
        assert!(result.is_ok());

        let content = result.unwrap();
        assert!(content.contains("description: Test rule"));
        assert!(content.contains("trigger: glob"));
        assert!(content.contains("globs: '**/*.rs'"));
        assert!(content.contains("# Test Rule"));
    }

    #[test]
    fn test_windsurf_processor_convert_to_agentsync() {
        use indoc::indoc;

        let processor = WindsurfProcessor;

        let windsurf_content = indoc! {r#"
            ---
            trigger: model_decision
            description: "Python development rules"
            globs: "**/*.py"
            ---

            # Python Rules

            Use type hints.
        "#};

        let result = processor.convert_to_agentsync(windsurf_content, "test.md");
        assert!(result.is_ok());

        let rule = result.unwrap();
        assert_eq!(rule.frontmatter.description, "Python development rules");
        // When converting from tool format, targets is set to "*" (all tools)
        assert!(rule.frontmatter.targets.contains(&"*".to_string()));
        assert!(rule.content.contains("# Python Rules"));
    }

    #[test]
    fn test_windsurf_processor_convert_roundtrip() {
        let processor = WindsurfProcessor;

        // Start with AgentSync format
        let original = Rule {
            frontmatter: AgentSyncRule {
                targets: vec!["windsurf".to_string()],
                description: "Roundtrip test".to_string(),
                globs: "**/*.ts".to_string(),
                cursor: None,
                windsurf: Some(WindsurfConfig {
                    trigger: WindsurfTrigger::Glob,
                    globs: "**/*.ts".to_string(),
                }),
                copilot: None,
            },
            content: "# Roundtrip\n\nTest content.".to_string(),
        };

        // Convert to Windsurf format
        let windsurf_content = processor.convert_from_agentsync(&original).unwrap();

        // Convert back to AgentSync format
        let converted = processor
            .convert_to_agentsync(&windsurf_content, "test.md")
            .unwrap();

        // Verify key fields are preserved
        assert_eq!(converted.frontmatter.description, original.frontmatter.description);
        assert!(converted.content.contains("Roundtrip"));
    }

    #[test]
    fn test_windsurf_processor_all_trigger_modes() {
        use indoc::formatdoc;

        let processor = WindsurfProcessor;

        let triggers = vec![
            ("manual", WindsurfTrigger::Manual),
            ("always_on", WindsurfTrigger::AlwaysOn),
            ("model_decision", WindsurfTrigger::ModelDecision),
            ("glob", WindsurfTrigger::Glob),
        ];

        for (trigger_str, _trigger_enum) in triggers {
            let content = formatdoc! {r#"
                ---
                trigger: {trigger_str}
                description: "Test"
                globs: "**/*"
                ---

                Content
            "#};

            let result = processor.convert_to_agentsync(&content, "test.md");
            assert!(
                result.is_ok(),
                "Failed to parse trigger mode: {trigger_str}"
            );
        }
    }

    #[test]
    fn test_windsurf_processor_invalid_frontmatter() {
        use indoc::indoc;

        let processor = WindsurfProcessor;

        let invalid_content = indoc! {r#"
            ---
            trigger: invalid_trigger
            description: "Test"
            ---

            Content
        "#};

        let result = processor.convert_to_agentsync(invalid_content, "test.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_windsurf_processor_missing_frontmatter() {
        let processor = WindsurfProcessor;

        let no_frontmatter = "# Just Content\n\nNo frontmatter here.";

        let result = processor.convert_to_agentsync(no_frontmatter, "test.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_windsurf_processor_empty_content() {
        use indoc::indoc;

        let processor = WindsurfProcessor;

        let empty_content = indoc! {r#"
            ---
            trigger: manual
            description: "Empty rule"
            ---
        "#};

        let result = processor.convert_to_agentsync(empty_content, "test.md");
        assert!(result.is_ok());

        let rule = result.unwrap();
        assert_eq!(rule.frontmatter.description, "Empty rule");
        assert!(rule.content.is_empty() || rule.content.trim().is_empty());
    }

    #[test]
    fn test_windsurf_processor_default_trigger() {
        use indoc::indoc;

        let processor = WindsurfProcessor;

        // Windsurf should have default trigger if not specified
        let content = indoc! {r#"
            ---
            description: "Test with default trigger"
            ---

            Content
        "#};

        let result = processor.convert_to_agentsync(content, "test.md");
        assert!(result.is_ok());

        let rule = result.unwrap();
        assert!(rule.frontmatter.windsurf.is_some());
        // Default trigger should be ModelDecision
        assert_eq!(
            rule.frontmatter.windsurf.unwrap().trigger,
            WindsurfTrigger::ModelDecision
        );
    }
}

