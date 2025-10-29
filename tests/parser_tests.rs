//! Integration tests for parsing fixture files

// Allow expect/unwrap in tests for brevity
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

mod common;

use agentsync::models::{AgentSyncRule, CopilotRule, CursorRule, WindsurfRule, WindsurfTrigger};
use agentsync::parser::parse_frontmatter;
use fs_err as fs;

/// Test parsing Cursor fixture with alwaysApply
#[test]
fn test_parse_cursor_always_apply() {
    let content = fs::read_to_string("tests/fixtures/cursor/python-dev.mdc")
        .expect("Failed to read cursor fixture");

    let rule = parse_frontmatter::<CursorRule>(&content, None).expect("Failed to parse");

    assert_eq!(
        rule.frontmatter.description,
        "General python development rules"
    );
    assert!(rule.frontmatter.always_apply);
    assert_eq!(rule.frontmatter.globs, "");
    assert!(rule.content.contains("Python Development"));
    assert!(rule.content.contains("Use type hints"));
}

/// Test parsing Cursor fixture with globs
#[test]
fn test_parse_cursor_with_globs() {
    let content = fs::read_to_string("tests/fixtures/cursor/react-components.mdc")
        .expect("Failed to read cursor fixture");

    let rule = parse_frontmatter::<CursorRule>(&content, None).expect("Failed to parse");

    assert_eq!(rule.frontmatter.description, "React component guidelines");
    assert!(!rule.frontmatter.always_apply);
    assert_eq!(rule.frontmatter.globs, "src/**/*.tsx, src/**/*.jsx");
    assert!(rule.content.contains("React Components"));
    assert!(rule.content.contains("functional components"));
}

/// Test parsing Copilot fixture
#[test]
fn test_parse_copilot() {
    let content = fs::read_to_string("tests/fixtures/copilot/python-standards.instructions.md")
        .expect("Failed to read copilot fixture");

    let rule = parse_frontmatter::<CopilotRule>(&content, None).expect("Failed to parse");

    assert_eq!(rule.frontmatter.description, "Python development standards");
    assert_eq!(rule.frontmatter.apply_to, "**/*.py");
    assert!(rule.content.contains("Python Standards"));
    assert!(rule.content.contains("docstrings"));
}

/// Test parsing Windsurf fixture
#[test]
fn test_parse_windsurf() {
    let content = fs::read_to_string("tests/fixtures/windsurf/python-dev.md")
        .expect("Failed to read windsurf fixture");

    let rule = parse_frontmatter::<WindsurfRule>(&content, None).expect("Failed to parse");

    assert_eq!(rule.frontmatter.trigger, WindsurfTrigger::ModelDecision);
    assert_eq!(
        rule.frontmatter.description,
        "General python development rules"
    );
    assert_eq!(
        rule.frontmatter.globs,
        "src/autopager/**/*.py, tests/**/*.py"
    );
    assert!(rule.content.contains("Python Development"));
    assert!(rule.content.contains("Clean Code"));
}

/// Test parsing AgentSync fixture with full configuration
#[test]
fn test_parse_agentsync() {
    let content = fs::read_to_string("tests/fixtures/agentsync/rust-dev.md")
        .expect("Failed to read agentsync fixture");

    let rule = parse_frontmatter::<AgentSyncRule>(&content, None).expect("Failed to parse");

    assert_eq!(rule.frontmatter.targets, vec!["*"]);
    assert_eq!(rule.frontmatter.description, "Comprehensive rule example");
    assert_eq!(rule.frontmatter.globs, "**/*.rs");

    // Check cursor config
    let cursor = rule
        .frontmatter
        .cursor
        .as_ref()
        .expect("Cursor config missing");
    assert!(!cursor.always_apply);
    assert_eq!(cursor.globs, "**/*.rs");

    // Check windsurf config
    let windsurf = rule
        .frontmatter
        .windsurf
        .as_ref()
        .expect("Windsurf config missing");
    assert_eq!(windsurf.trigger, WindsurfTrigger::Glob);
    assert_eq!(windsurf.globs, "**/*.rs");

    // Check copilot config
    let copilot = rule
        .frontmatter
        .copilot
        .as_ref()
        .expect("Copilot config missing");
    assert_eq!(copilot.apply_to, "**/*.rs");

    assert!(rule.content.contains("Rust Development"));
    assert!(rule.content.contains("best practices"));
}

/// Test parsing invalid frontmatter (no opening delimiter)
#[test]
fn test_parse_no_opening_delimiter() {
    let content = "# Just markdown\n\nNo frontmatter here";
    let result = parse_frontmatter::<CursorRule>(content, None);
    assert!(result.is_err());
}

/// Test parsing invalid frontmatter (no closing delimiter)
#[test]
fn test_parse_no_closing_delimiter() {
    let content = "---\ndescription: Test\nalwaysApply: true\n\nNo closing delimiter";
    let result = parse_frontmatter::<CursorRule>(content, None);
    assert!(result.is_err());
}

/// Test parsing invalid YAML
#[test]
fn test_parse_invalid_yaml() {
    let content = r"---
description: Test
alwaysApply: [invalid yaml structure
---

Content
";
    let result = parse_frontmatter::<CursorRule>(content, None);
    assert!(result.is_err());
}

/// Test parsing with extra whitespace
#[test]
fn test_parse_with_whitespace() {
    let content = r#"

---
description: "Test with whitespace"
alwaysApply: true
globs: ""
---


# Content with spacing

Test
"#;

    let rule = parse_frontmatter::<CursorRule>(content, None).expect("Failed to parse");
    assert_eq!(rule.frontmatter.description, "Test with whitespace");
    assert!(rule.frontmatter.always_apply);
}

/// Test parsing minimal frontmatter (using defaults)
#[test]
fn test_parse_minimal_frontmatter() {
    let content = r#"---
description: "Minimal"
---

# Content
"#;

    let rule = parse_frontmatter::<CursorRule>(content, None).expect("Failed to parse");
    assert_eq!(rule.frontmatter.description, "Minimal");
    assert!(!rule.frontmatter.always_apply); // Default is false
    assert_eq!(rule.frontmatter.globs, ""); // Default is empty
}

/// Test roundtrip: parse and serialize back
#[test]
fn test_roundtrip_parsing() {
    use agentsync::parser::serialize_frontmatter;

    let original =
        fs::read_to_string("tests/fixtures/cursor/python-dev.mdc").expect("Failed to read fixture");

    let rule = parse_frontmatter::<CursorRule>(&original, None).expect("Failed to parse");
    let serialized = serialize_frontmatter(&rule).expect("Failed to serialize");
    let reparsed = parse_frontmatter::<CursorRule>(&serialized, None).expect("Failed to reparse");

    assert_eq!(
        rule.frontmatter.description,
        reparsed.frontmatter.description
    );
    assert_eq!(
        rule.frontmatter.always_apply,
        reparsed.frontmatter.always_apply
    );
    assert_eq!(rule.frontmatter.globs, reparsed.frontmatter.globs);
    assert_eq!(rule.content.trim(), reparsed.content.trim());
}
