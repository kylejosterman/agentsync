//! Frontmatter parsing for rule files
//!
//! This module handles parsing markdown files with YAML frontmatter.
//! Format: YAML between `---` delimiters, followed by markdown content.
//!
//! Example:
//! ```text
//! ---
//! description: "Rule description"
//! alwaysApply: true
//! ---
//!
//! # Rule Content
//! Markdown content here...
//! ```

use crate::models::Rule;
use crate::{AgentSyncError, Result};
use serde::de::DeserializeOwned;

/// Parse a markdown file with YAML frontmatter
///
/// Splits the file into frontmatter (YAML between ---) and content (markdown body)
///
/// # Arguments
/// * `content` - The file content to parse
/// * `filename` - Optional filename for better error messages
pub fn parse_frontmatter<T: DeserializeOwned>(
    content: &str,
    filename: Option<&str>,
) -> Result<Rule<T>> {
    let (frontmatter_str, body, start_line) = split_frontmatter(content, filename)?;

    // Parse YAML with line number context
    let frontmatter: T = serde_yaml::from_str(&frontmatter_str).map_err(|e| {
        // Try to extract line number from YAML error if available
        let line = e.location().map(|loc| start_line + loc.line());
        AgentSyncError::invalid_frontmatter(filename.unwrap_or("unknown"), line, e)
    })?;

    Ok(Rule {
        frontmatter,
        content: body,
    })
}

/// Split frontmatter from markdown content
///
/// Returns `(frontmatter_yaml, markdown_body, start_line)`
fn split_frontmatter(content: &str, filename: Option<&str>) -> Result<(String, String, usize)> {
    let content = content.trim_start();
    let file = filename.unwrap_or("unknown").to_string();

    // Check if file starts with ---
    if !content.starts_with("---") {
        // Create a synthetic YAML error for consistency
        #[allow(clippy::expect_used)]
        let yaml_err =
            serde_yaml::from_str::<()>("invalid: [").expect_err("synthetic YAML error should fail");
        return Err(AgentSyncError::invalid_frontmatter(file, Some(1), yaml_err));
    }

    // Find the closing --- delimiter
    let after_first = &content[3..]; // Skip first ---

    if let Some(end_pos) = after_first.find("\n---") {
        // Extract frontmatter (between the two --- markers)
        let frontmatter = after_first[..end_pos].trim().to_string();

        // Extract body (everything after the second ---)
        let body_start = end_pos + 4; // Skip \n---
        let body = if body_start < after_first.len() {
            after_first[body_start..].trim_start().to_string()
        } else {
            String::new()
        };

        // Start line is 2 (line after first ---)
        Ok((frontmatter, body, 2))
    } else {
        // Create a synthetic YAML error for consistency
        #[allow(clippy::expect_used)]
        let yaml_err =
            serde_yaml::from_str::<()>("invalid: [").expect_err("synthetic YAML error should fail");
        Err(AgentSyncError::invalid_frontmatter(file, None, yaml_err))
    }
}

/// Serialize frontmatter and content back to markdown file format
pub fn serialize_frontmatter<T: serde::Serialize>(rule: &Rule<T>) -> Result<String> {
    let frontmatter_yaml = serde_yaml::to_string(&rule.frontmatter)?;

    // Build the complete file content
    let mut result = String::from("---\n");
    result.push_str(&frontmatter_yaml);
    // serde_yaml::to_string already adds trailing newline
    result.push_str("---\n\n");
    result.push_str(&rule.content);

    // Ensure file ends with newline
    if !result.ends_with('\n') {
        result.push('\n');
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    // Allow expect/unwrap in tests for brevity
    #![allow(clippy::expect_used)]
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::models::{CopilotRule, CursorRule, WindsurfRule, WindsurfTrigger};

    #[test]
    fn test_split_frontmatter_valid() {
        let content = r"---
description: Test rule
alwaysApply: true
---

# Test Content

This is the body.
";

        let (frontmatter, body, start_line) =
            split_frontmatter(content, None).expect("should parse valid frontmatter");
        assert!(frontmatter.contains("description: Test rule"));
        assert!(frontmatter.contains("alwaysApply: true"));
        assert!(body.starts_with("# Test Content"));
        assert_eq!(start_line, 2);
    }

    #[test]
    fn test_split_frontmatter_no_opening() {
        let content = "# Just markdown\n\nNo frontmatter";
        let result = split_frontmatter(content, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_split_frontmatter_no_closing() {
        let content = "---\ndescription: Test\n\nNo closing delimiter";
        let result = split_frontmatter(content, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_cursor_frontmatter() {
        let content = r"---
description: 'Python development rules'
alwaysApply: true
globs: ''
---

# Python Rules

Use type hints for all functions.
";

        let rule: Rule<CursorRule> =
            parse_frontmatter(content, None).expect("should parse cursor frontmatter");
        assert_eq!(rule.frontmatter.description, "Python development rules");
        assert!(rule.frontmatter.always_apply);
        assert_eq!(rule.frontmatter.globs, "");
        assert!(rule.content.contains("Python Rules"));
    }

    #[test]
    fn test_parse_windsurf_frontmatter() {
        let content = r"---
trigger: model_decision
description: 'General python development'
globs: 'src/**/*.py, tests/**/*.py'
---

# Python Development

Follow best practices.
";

        let rule: Rule<WindsurfRule> =
            parse_frontmatter(content, None).expect("should parse windsurf frontmatter");
        assert_eq!(rule.frontmatter.trigger, WindsurfTrigger::ModelDecision);
        assert_eq!(rule.frontmatter.description, "General python development");
        assert_eq!(rule.frontmatter.globs, "src/**/*.py, tests/**/*.py");
        assert!(rule.content.contains("Python Development"));
    }

    #[test]
    fn test_parse_copilot_frontmatter() {
        let content = r"---
description: 'Python standards'
applyTo: '**/*.py'
---

# Python Standards

Write docstrings.
";

        let rule: Rule<CopilotRule> =
            parse_frontmatter(content, None).expect("should parse copilot frontmatter");
        assert_eq!(rule.frontmatter.description, "Python standards");
        assert_eq!(rule.frontmatter.apply_to, "**/*.py");
        assert!(rule.content.contains("Python Standards"));
    }

    #[test]
    fn test_serialize_frontmatter() {
        let rule = Rule {
            frontmatter: CursorRule {
                description: "Test rule".to_string(),
                always_apply: true,
                globs: String::new(),
            },
            content: "# Test\n\nContent here.".to_string(),
        };

        let serialized = serialize_frontmatter(&rule).expect("should serialize frontmatter");

        assert!(serialized.starts_with("---\n"));
        assert!(serialized.contains("description: Test rule"));
        assert!(serialized.contains("alwaysApply: true"));
        assert!(serialized.contains("---\n\n# Test"));
        assert!(serialized.ends_with('\n'));
    }

    #[test]
    fn test_roundtrip() {
        let original = r"---
description: 'Roundtrip test'
alwaysApply: false
globs: '**/*.rs'
---

# Rust Rules

Use idiomatic patterns.
";

        // Parse
        let rule: Rule<CursorRule> =
            parse_frontmatter(original, None).expect("should parse original");

        // Verify parsed data
        assert_eq!(rule.frontmatter.description, "Roundtrip test");
        assert!(!rule.frontmatter.always_apply);
        assert_eq!(rule.frontmatter.globs, "**/*.rs");

        // Serialize back
        let serialized = serialize_frontmatter(&rule).expect("should serialize");

        // Parse again
        let rule2: Rule<CursorRule> =
            parse_frontmatter(&serialized, None).expect("should parse serialized");

        // Verify data is preserved
        assert_eq!(rule.frontmatter.description, rule2.frontmatter.description);
        assert_eq!(
            rule.frontmatter.always_apply,
            rule2.frontmatter.always_apply
        );
        assert_eq!(rule.frontmatter.globs, rule2.frontmatter.globs);
        assert_eq!(rule.content.trim(), rule2.content.trim());
    }
}
