//! Copilot tool processor implementation

use super::Processor;
use crate::converter::{agentsync_rule_to_copilot, copilot_rule_to_agentsync};
use crate::fs::Tool;
use crate::models::{AgentSyncRule, CopilotRule, Rule};
use crate::parser::{parse_frontmatter, serialize_frontmatter};
use crate::Result;

/// Processor for GitHub Copilot (.md files in .github/instructions/)
pub struct CopilotProcessor;

impl Processor for CopilotProcessor {
    fn tool(&self) -> Tool {
        Tool::Copilot
    }

    fn convert_from_agentsync(&self, rule: &Rule<AgentSyncRule>) -> Result<String> {
        let copilot_rule = agentsync_rule_to_copilot(rule);
        serialize_frontmatter(&copilot_rule)
    }

    fn convert_to_agentsync(&self, content: &str, path: &str) -> Result<Rule<AgentSyncRule>> {
        let copilot_rule: Rule<CopilotRule> = parse_frontmatter(content, Some(path))?;
        Ok(copilot_rule_to_agentsync(&copilot_rule))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CopilotConfig;

    #[test]
    fn test_copilot_processor_tool() {
        let processor = CopilotProcessor;
        assert_eq!(processor.tool(), Tool::Copilot);
    }

    #[test]
    fn test_copilot_processor_convert_from_agentsync() {
        let processor = CopilotProcessor;

        let agentsync_rule = Rule {
            frontmatter: AgentSyncRule {
                targets: vec!["copilot".to_string()],
                description: "Test rule".to_string(),
                globs: "**/*.rs".to_string(),
                cursor: None,
                windsurf: None,
                copilot: Some(CopilotConfig {
                    apply_to: "**/*.rs".to_string(),
                }),
            },
            content: "# Test Rule\n\nThis is a test.".to_string(),
        };

        let result = processor.convert_from_agentsync(&agentsync_rule);
        assert!(result.is_ok());

        let content = result.unwrap();
        assert!(content.contains("description: Test rule"));
        assert!(content.contains("applyTo:"));
        assert!(content.contains("# Test Rule"));
    }

    #[test]
    fn test_copilot_processor_convert_to_agentsync() {
        use indoc::indoc;

        let processor = CopilotProcessor;

        let copilot_content = indoc! {r#"
            ---
            description: "JavaScript standards"
            applyTo: "**/*.js"
            ---

            # JavaScript Rules

            Use strict mode.
        "#};

        let result = processor.convert_to_agentsync(copilot_content, "test.md");
        assert!(result.is_ok());

        let rule = result.unwrap();
        assert_eq!(rule.frontmatter.description, "JavaScript standards");
        // When converting from tool format, targets is set to "*" (all tools)
        assert!(rule.frontmatter.targets.contains(&"*".to_string()));
        assert!(rule.content.contains("# JavaScript Rules"));
    }

    #[test]
    fn test_copilot_processor_convert_roundtrip() {
        let processor = CopilotProcessor;

        // Start with AgentSync format
        let original = Rule {
            frontmatter: AgentSyncRule {
                targets: vec!["copilot".to_string()],
                description: "Roundtrip test".to_string(),
                globs: "**/*.go".to_string(),
                cursor: None,
                windsurf: None,
                copilot: Some(CopilotConfig {
                    apply_to: "**/*.go".to_string(),
                }),
            },
            content: "# Roundtrip\n\nTest content.".to_string(),
        };

        // Convert to Copilot format
        let copilot_content = processor.convert_from_agentsync(&original).unwrap();

        // Convert back to AgentSync format
        let converted = processor
            .convert_to_agentsync(&copilot_content, "test.md")
            .unwrap();

        // Verify key fields are preserved
        assert_eq!(converted.frontmatter.description, original.frontmatter.description);
        assert!(converted.content.contains("Roundtrip"));
    }

    #[test]
    fn test_copilot_processor_default_apply_to() {
        use indoc::indoc;

        let processor = CopilotProcessor;

        let copilot_content = indoc! {r#"
            ---
            description: "Default applyTo test"
            ---

            # Test

            Content
        "#};

        let result = processor.convert_to_agentsync(copilot_content, "test.md");
        assert!(result.is_ok());

        let rule = result.unwrap();
        // Should have default applyTo value
        assert!(rule.frontmatter.copilot.is_some());
    }

    #[test]
    fn test_copilot_processor_invalid_frontmatter() {
        use indoc::indoc;

        let processor = CopilotProcessor;

        let invalid_content = indoc! {r#"
            ---
            description: 123
            applyTo: ["not", "a", "string"]
            ---

            Content
        "#};

        let result = processor.convert_to_agentsync(invalid_content, "test.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_copilot_processor_missing_frontmatter() {
        let processor = CopilotProcessor;

        let no_frontmatter = "# Just Content\n\nNo frontmatter here.";

        let result = processor.convert_to_agentsync(no_frontmatter, "test.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_copilot_processor_empty_content() {
        use indoc::indoc;

        let processor = CopilotProcessor;

        let empty_content = indoc! {r#"
            ---
            description: "Empty rule"
            applyTo: "**"
            ---
        "#};

        let result = processor.convert_to_agentsync(empty_content, "test.md");
        assert!(result.is_ok());

        let rule = result.unwrap();
        assert_eq!(rule.frontmatter.description, "Empty rule");
        assert!(rule.content.is_empty() || rule.content.trim().is_empty());
    }
}

