//! Parse markdown files with YAML frontmatter between `---` delimiters.

use crate::models::{
    AgentSyncRule, CopilotConfig, CopilotRule, CursorConfig, CursorRule, Rule, WindsurfConfig,
    WindsurfRule, WindsurfTrigger,
};
use crate::{AgentSyncError, Result};
use std::collections::HashMap;
use std::str::FromStr;

/// Trait for parsing frontmatter from key-value pairs
pub trait ParseFrontmatter: Sized {
    fn from_key_values(map: &HashMap<String, String>) -> Result<Self>;
}

/// Trait for serializing frontmatter to key-value pairs
pub trait SerializeFrontmatter {
    fn to_key_values(&self) -> Vec<(String, String)>;
}

/// Split frontmatter from markdown. Returns `(frontmatter_text, body)`.
fn split_frontmatter(content: &str, filename: Option<&str>) -> Result<(String, String)> {
    let content = content.trim_start();
    let file = filename.unwrap_or("unknown");

    // Check if file starts with ---
    if !content.starts_with("---") {
        return Err(AgentSyncError::FrontmatterParse {
            file: file.to_string(),
            line: Some(1),
            message: "Missing opening '---' delimiter".to_string(),
        });
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

        Ok((frontmatter, body))
    } else {
        Err(AgentSyncError::FrontmatterParse {
            file: file.to_string(),
            line: None,
            message: "Missing closing '---' delimiter".to_string(),
        })
    }
}

/// Remove surrounding quotes
fn unquote(s: &str) -> &str {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        if s.len() >= 2 { &s[1..s.len() - 1] } else { s }
    } else {
        s
    }
}

/// Parse JSON array notation to comma-separated string
fn parse_json_array(value: &str) -> String {
    if !value.starts_with('[') || !value.ends_with(']') {
        return value.to_string();
    }

    let inner = &value[1..value.len() - 1];
    inner
        .split(',')
        .map(unquote)
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join(",")
}

/// Join list items and insert into map
fn finalize_list_items(
    map: &mut HashMap<String, String>,
    parent: Option<&String>,
    items: &mut Vec<String>,
) {
    if !items.is_empty() {
        if let Some(p) = parent {
            map.insert(p.clone(), items.join(","));
        }
        items.clear();
    }
}

/// Parse key-value pairs from frontmatter (supports nesting, lists, JSON arrays)
fn parse_key_value_pairs(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut current_parent: Option<String> = None;
    let mut list_items: Vec<String> = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let indent_level = line.len() - line.trim_start().len();

        // Handle YAML list items (- value)
        if trimmed.starts_with('-') && indent_level > 0 {
            if current_parent.is_some() {
                let item = unquote(&trimmed[1..]);
                list_items.push(item.to_string());
            }
            continue;
        }

        // Finalize any pending list items when we encounter a non-list line
        if !list_items.is_empty() && !trimmed.starts_with('-') {
            finalize_list_items(&mut map, current_parent.as_ref(), &mut list_items);
            current_parent = None;
        }

        // Split on first colon
        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim();
            let value = unquote(value);

            if indent_level == 0 {
                // Top-level key: handle JSON arrays and regular values
                let parsed_value = parse_json_array(value);
                map.insert(key.to_string(), parsed_value);

                // Track parent for nested values or lists
                current_parent = if value.is_empty() {
                    Some(key.to_string())
                } else {
                    None
                };
            } else if let Some(ref parent) = current_parent {
                // Nested key under parent
                let nested_key = format!("{parent}:{key}");
                map.insert(nested_key, value.to_string());
            }
        }
    }

    // Finalize any remaining list items at the end
    finalize_list_items(&mut map, current_parent.as_ref(), &mut list_items);

    map
}

/// Normalize glob patterns by removing spaces after commas
fn normalize_globs(globs: &str) -> String {
    if globs.is_empty() {
        return String::new();
    }
    globs
        .split(',')
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(",")
}

/// Parse bool from string with fallback
fn parse_bool(value: &str, default: bool) -> bool {
    match value.to_lowercase().as_str() {
        "true" => true,
        "false" => false,
        _ => default,
    }
}

impl ParseFrontmatter for CursorRule {
    fn from_key_values(map: &HashMap<String, String>) -> Result<Self> {
        Ok(Self {
            description: map.get("description").cloned().unwrap_or_default(),
            always_apply: parse_bool(map.get("alwaysApply").map_or("", String::as_str), false),
            globs: normalize_globs(map.get("globs").map_or("", String::as_str)),
        })
    }
}

impl SerializeFrontmatter for CursorRule {
    fn to_key_values(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if !self.description.is_empty() {
            pairs.push(("description".to_string(), self.description.clone()));
        }
        pairs.push(("alwaysApply".to_string(), self.always_apply.to_string()));
        if !self.globs.is_empty() {
            pairs.push(("globs".to_string(), self.globs.clone()));
        }
        pairs
    }
}

impl ParseFrontmatter for WindsurfRule {
    fn from_key_values(map: &HashMap<String, String>) -> Result<Self> {
        let trigger = map
            .get("trigger")
            .and_then(|s| WindsurfTrigger::from_str(s).ok())
            .unwrap_or_default();

        Ok(Self {
            trigger,
            description: map.get("description").cloned().unwrap_or_default(),
            globs: normalize_globs(map.get("globs").map_or("", String::as_str)),
        })
    }
}

impl SerializeFrontmatter for WindsurfRule {
    fn to_key_values(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        pairs.push(("trigger".to_string(), self.trigger.to_string()));
        if !self.description.is_empty() {
            pairs.push(("description".to_string(), self.description.clone()));
        }
        if !self.globs.is_empty() {
            pairs.push(("globs".to_string(), self.globs.clone()));
        }
        pairs
    }
}

impl ParseFrontmatter for CopilotRule {
    fn from_key_values(map: &HashMap<String, String>) -> Result<Self> {
        Ok(Self {
            description: map.get("description").cloned().unwrap_or_default(),
            apply_to: normalize_globs(map.get("applyTo").map_or("**", String::as_str)),
        })
    }
}

impl SerializeFrontmatter for CopilotRule {
    fn to_key_values(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if !self.description.is_empty() {
            pairs.push(("description".to_string(), self.description.clone()));
        }
        pairs.push(("applyTo".to_string(), self.apply_to.clone()));
        pairs
    }
}

impl ParseFrontmatter for AgentSyncRule {
    fn from_key_values(map: &HashMap<String, String>) -> Result<Self> {
        // Parse targets array
        let targets = map.get("targets").map_or_else(
            || vec!["*".to_string()],
            |s| {
                s.split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect()
            },
        );

        // Parse nested cursor config
        let cursor = if map.contains_key("cursor") {
            let always_apply = parse_bool(
                map.get("cursor:alwaysApply").map_or("", String::as_str),
                false,
            );
            let globs = normalize_globs(map.get("cursor:globs").map_or("", String::as_str));
            Some(CursorConfig {
                always_apply,
                globs,
            })
        } else {
            None
        };

        // Parse nested windsurf config
        let windsurf = if map.contains_key("windsurf") {
            let trigger = map
                .get("windsurf:trigger")
                .and_then(|s| WindsurfTrigger::from_str(s).ok())
                .unwrap_or_default();
            let globs = normalize_globs(map.get("windsurf:globs").map_or("", String::as_str));
            Some(WindsurfConfig { trigger, globs })
        } else {
            None
        };

        // Parse nested copilot config
        let copilot = if map.contains_key("copilot") {
            let apply_to = normalize_globs(map.get("copilot:applyTo").map_or("**", String::as_str));
            Some(CopilotConfig { apply_to })
        } else {
            None
        };

        Ok(Self {
            targets,
            description: map.get("description").cloned().unwrap_or_default(),
            globs: normalize_globs(map.get("globs").map_or("**/*", String::as_str)),
            cursor,
            windsurf,
            copilot,
        })
    }
}

impl SerializeFrontmatter for AgentSyncRule {
    fn to_key_values(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        // Targets
        pairs.push(("targets".to_string(), self.targets.join(",")));

        // Description
        if !self.description.is_empty() {
            pairs.push(("description".to_string(), self.description.clone()));
        }

        // Globs
        pairs.push(("globs".to_string(), self.globs.clone()));

        // Nested cursor config
        if let Some(ref cursor) = self.cursor {
            pairs.push(("cursor".to_string(), String::new()));
            pairs.push((
                "cursor:alwaysApply".to_string(),
                cursor.always_apply.to_string(),
            ));
            pairs.push(("cursor:globs".to_string(), cursor.globs.clone()));
        }

        // Nested windsurf config
        if let Some(ref windsurf) = self.windsurf {
            pairs.push(("windsurf".to_string(), String::new()));
            pairs.push(("windsurf:trigger".to_string(), windsurf.trigger.to_string()));
            pairs.push(("windsurf:globs".to_string(), windsurf.globs.clone()));
        }

        // Nested copilot config
        if let Some(ref copilot) = self.copilot {
            pairs.push(("copilot".to_string(), String::new()));
            pairs.push(("copilot:applyTo".to_string(), copilot.apply_to.clone()));
        }

        pairs
    }
}

/// Parse markdown file with frontmatter
pub fn parse_frontmatter<T: ParseFrontmatter>(
    content: &str,
    filename: Option<&str>,
) -> Result<Rule<T>> {
    let (frontmatter_str, body) = split_frontmatter(content, filename)?;
    let map = parse_key_value_pairs(&frontmatter_str);
    let frontmatter = T::from_key_values(&map)?;

    Ok(Rule {
        frontmatter,
        content: body,
    })
}

/// Serialize frontmatter and content to markdown
pub fn serialize_frontmatter<T: SerializeFrontmatter>(rule: &Rule<T>) -> Result<String> {
    let pairs = rule.frontmatter.to_key_values();

    let mut result = String::from("---\n");

    for (key, value) in pairs {
        if key.contains(':') {
            // Nested key - add indentation
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() == 2 {
                result.push_str("  ");
                result.push_str(parts[1]);
                result.push_str(": ");
                result.push_str(&value);
                result.push('\n');
            }
        } else if value.is_empty() {
            // Parent key with no value (for nested structures)
            result.push_str(&key);
            result.push_str(":\n");
        } else {
            // Regular key-value pair
            result.push_str(&key);
            result.push_str(": ");
            result.push_str(&value);
            result.push('\n');
        }
    }

    result.push_str("---\n");
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

    #[test]
    fn test_split_frontmatter_valid() {
        let content = r"---
description: Test rule
alwaysApply: true
---

# Test Content

This is the body.
";

        let (frontmatter, body) =
            split_frontmatter(content, None).expect("should parse valid frontmatter");
        assert!(frontmatter.contains("description: Test rule"));
        assert!(frontmatter.contains("alwaysApply: true"));
        assert!(body.starts_with("# Test Content"));
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
description: Python development rules
alwaysApply: true
globs:
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
    fn test_parse_cursor_frontmatter_unquoted_globs() {
        let content = r"---
description: Python rules
alwaysApply: false
globs: **/*.py,**/*.pyi
---

# Python
";

        let rule: Rule<CursorRule> =
            parse_frontmatter(content, None).expect("should parse unquoted globs");
        assert_eq!(rule.frontmatter.globs, "**/*.py,**/*.pyi");
    }

    #[test]
    fn test_parse_windsurf_frontmatter() {
        let content = r"---
trigger: model_decision
description: General python development
globs: src/**/*.py,tests/**/*.py
---

# Python Development

Follow best practices.
";

        let rule: Rule<WindsurfRule> =
            parse_frontmatter(content, None).expect("should parse windsurf frontmatter");
        assert_eq!(rule.frontmatter.trigger, WindsurfTrigger::ModelDecision);
        assert_eq!(rule.frontmatter.description, "General python development");
        assert_eq!(rule.frontmatter.globs, "src/**/*.py,tests/**/*.py");
        assert!(rule.content.contains("Python Development"));
    }

    #[test]
    fn test_parse_copilot_frontmatter() {
        let content = r"---
description: Python standards
applyTo: **/*.py
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
    fn test_serialize_cursor_frontmatter() {
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
        assert!(serialized.contains("---\n# Test"));
        assert!(serialized.ends_with('\n'));
        // Ensure globs are not quoted
        assert!(!serialized.contains('"'));
    }

    #[test]
    fn test_serialize_cursor_with_globs() {
        let rule = Rule {
            frontmatter: CursorRule {
                description: "Rust rule".to_string(),
                always_apply: false,
                globs: "**/*.rs,**/*.toml".to_string(),
            },
            content: "# Rust\n".to_string(),
        };

        let serialized = serialize_frontmatter(&rule).expect("should serialize");

        assert!(serialized.contains("globs: **/*.rs,**/*.toml"));
        // Ensure no quotes around globs
        assert!(!serialized.contains('"'));
    }

    #[test]
    fn test_roundtrip_cursor() {
        let original = r"---
description: Roundtrip test
alwaysApply: false
globs: **/*.rs
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

    #[test]
    fn test_unquote() {
        assert_eq!(unquote("\"hello\""), "hello");
        assert_eq!(unquote("'hello'"), "hello");
        assert_eq!(unquote("hello"), "hello");
        assert_eq!(unquote("  \"hello\"  "), "hello");
        assert_eq!(unquote("\""), "\"");
        assert_eq!(unquote(""), "");
    }

    #[test]
    fn test_parse_json_array() {
        assert_eq!(parse_json_array("[\"*\"]"), "*");
        assert_eq!(
            parse_json_array("[\"cursor\", \"windsurf\"]"),
            "cursor,windsurf"
        );
        assert_eq!(parse_json_array("['a','b','c']"), "a,b,c");
        assert_eq!(parse_json_array("[\"a\", \"b\", \"c\"]"), "a,b,c");
        assert_eq!(parse_json_array("not-an-array"), "not-an-array");
        assert_eq!(parse_json_array("[]"), "");
    }

    #[test]
    fn test_normalize_globs() {
        assert_eq!(normalize_globs("**/*.py, **/*.pyi"), "**/*.py,**/*.pyi");
        assert_eq!(normalize_globs("**/*.py,**/*.pyi"), "**/*.py,**/*.pyi");
        assert_eq!(normalize_globs("**/*.py"), "**/*.py");
        assert_eq!(normalize_globs(""), "");
    }

    #[test]
    fn test_parse_agentsync_nested() {
        let content = r"---
targets: *
description: Test rule
globs: **/*.rs
cursor:
  alwaysApply: false
  globs: **/*.rs
windsurf:
  trigger: glob
  globs: **/*.rs
copilot:
  applyTo: **/*.rs
---

# Test
";

        let rule: Rule<AgentSyncRule> =
            parse_frontmatter(content, None).expect("should parse agentsync");
        assert_eq!(rule.frontmatter.targets, vec!["*"]);
        assert_eq!(rule.frontmatter.globs, "**/*.rs");

        let cursor = rule.frontmatter.cursor.expect("should have cursor config");
        assert!(!cursor.always_apply);
        assert_eq!(cursor.globs, "**/*.rs");

        let windsurf = rule
            .frontmatter
            .windsurf
            .expect("should have windsurf config");
        assert_eq!(windsurf.trigger, WindsurfTrigger::Glob);
        assert_eq!(windsurf.globs, "**/*.rs");

        let copilot = rule
            .frontmatter
            .copilot
            .expect("should have copilot config");
        assert_eq!(copilot.apply_to, "**/*.rs");
    }

    #[test]
    fn test_roundtrip_agentsync() {
        let rule = Rule {
            frontmatter: AgentSyncRule {
                targets: vec!["*".to_string()],
                description: "Test".to_string(),
                globs: "**/*.rs".to_string(),
                cursor: Some(CursorConfig {
                    always_apply: false,
                    globs: "**/*.rs".to_string(),
                }),
                windsurf: Some(WindsurfConfig {
                    trigger: WindsurfTrigger::Glob,
                    globs: "**/*.rs".to_string(),
                }),
                copilot: Some(CopilotConfig {
                    apply_to: "**/*.rs".to_string(),
                }),
            },
            content: "# Test\n".to_string(),
        };

        let serialized = serialize_frontmatter(&rule).expect("should serialize");

        // Verify no quotes around globs
        assert!(!serialized.contains('"'));
        assert!(serialized.contains("globs: **/*.rs"));

        // Parse back
        let rule2: Rule<AgentSyncRule> =
            parse_frontmatter(&serialized, None).expect("should parse back");

        assert_eq!(rule.frontmatter.globs, rule2.frontmatter.globs);
        assert_eq!(
            rule.frontmatter.cursor.as_ref().unwrap().globs,
            rule2.frontmatter.cursor.as_ref().unwrap().globs
        );
    }
}
