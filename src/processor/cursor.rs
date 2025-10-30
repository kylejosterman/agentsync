//! Cursor tool processor implementation

use super::Processor;
use crate::Result;
use crate::converter::{agentsync_rule_to_cursor, cursor_rule_to_agentsync};
use crate::fs::Tool;
use crate::models::{AgentSyncRule, CursorRule, Rule};
use crate::parser::{parse_frontmatter, serialize_frontmatter};

/// Processor for Cursor
pub struct CursorProcessor;

impl Processor for CursorProcessor {
    fn tool(&self) -> Tool {
        Tool::Cursor
    }

    fn convert_from_agentsync(&self, rule: &Rule<AgentSyncRule>) -> Result<String> {
        let cursor_rule = agentsync_rule_to_cursor(rule);
        serialize_frontmatter(&cursor_rule)
    }

    fn convert_to_agentsync(&self, content: &str, path: &str) -> Result<Rule<AgentSyncRule>> {
        let cursor_rule: Rule<CursorRule> = parse_frontmatter(content, Some(path))?;
        Ok(cursor_rule_to_agentsync(&cursor_rule))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CursorConfig;

    #[test]
    fn test_cursor_processor_convert_from_agentsync() {
        let processor = CursorProcessor;

        let agentsync_rule = Rule {
            frontmatter: AgentSyncRule {
                targets: vec!["cursor".to_string()],
                description: "Test rule".to_string(),
                globs: "**/*.rs".to_string(),
                cursor: Some(CursorConfig {
                    always_apply: false, // Use false to test description in frontmatter
                    globs: "**/*.rs".to_string(),
                }),
                windsurf: None,
                copilot: None,
            },
            content: "# Test Rule\n\nThis is a test.".to_string(),
        };

        let result = processor.convert_from_agentsync(&agentsync_rule);
        assert!(result.is_ok());

        let content = result.unwrap();
        assert!(content.contains("description: Test rule"));
        assert!(content.contains("alwaysApply: false"));
        assert!(content.contains("globs: **/*.rs"));
        assert!(content.contains("# Test Rule"));
    }

    #[test]
    fn test_cursor_processor_convert_to_agentsync() {
        use indoc::indoc;

        let processor = CursorProcessor;

        let cursor_content = indoc! {r#"
            ---
            description: "Python development rules"
            alwaysApply: false
            globs: "**/*.py"
            ---

            # Python Rules

            Use type hints.
        "#};

        let result = processor.convert_to_agentsync(cursor_content, "test.mdc");
        assert!(result.is_ok());

        let rule = result.unwrap();
        assert_eq!(rule.frontmatter.description, "Python development rules");
        // When converting from tool format, targets is set to "*" (all tools)
        assert!(rule.frontmatter.targets.contains(&"*".to_string()));
        assert!(rule.content.contains("# Python Rules"));
    }

    #[test]
    fn test_cursor_processor_convert_roundtrip() {
        let processor = CursorProcessor;

        // Start with AgentSync format
        let original = Rule {
            frontmatter: AgentSyncRule {
                targets: vec!["cursor".to_string()],
                description: "Roundtrip test".to_string(),
                globs: "**/*.ts".to_string(),
                cursor: Some(CursorConfig {
                    always_apply: false,
                    globs: "**/*.ts".to_string(),
                }),
                windsurf: None,
                copilot: None,
            },
            content: "# Roundtrip\n\nTest content.".to_string(),
        };

        // Convert to Cursor format
        let cursor_content = processor.convert_from_agentsync(&original).unwrap();

        // Convert back to AgentSync format
        let converted = processor
            .convert_to_agentsync(&cursor_content, "test.mdc")
            .unwrap();

        // Verify key fields are preserved
        assert_eq!(
            converted.frontmatter.description,
            original.frontmatter.description
        );
        assert!(converted.content.contains("Roundtrip"));
    }
}
