//! Integration tests for format conversion
//!
//! Tests conversion logic using actual fixture files

// Allow expect/unwrap in tests for brevity
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

mod common;

use agentsync::converter::{
    agentsync_rule_to_copilot, agentsync_rule_to_cursor, agentsync_rule_to_windsurf,
    copilot_rule_to_agentsync, cursor_rule_to_agentsync, windsurf_rule_to_agentsync,
};
use agentsync::models::{
    AgentSyncRule, CopilotRule, CursorRule, Rule, WindsurfRule, WindsurfTrigger,
};
use agentsync::parser::{parse_frontmatter, serialize_frontmatter};

const CURSOR_REACT_FIXTURE: &str = include_str!("fixtures/cursor/react-components.mdc");
const COPILOT_PYTHON_FIXTURE: &str = include_str!("fixtures/copilot/python-standards.instructions.md");
const WINDSURF_PYTHON_FIXTURE: &str = include_str!("fixtures/windsurf/python-dev.md");
const AGENTSYNC_RUST_FIXTURE: &str = include_str!("fixtures/agentsync/rust-dev.md");

// ============================================================================
// Cursor Conversion Integration Tests
// ============================================================================

#[test]
fn test_cursor_fixture_to_agentsync() {
    let cursor_rule: Rule<CursorRule> = parse_frontmatter(CURSOR_REACT_FIXTURE, None).unwrap();

    assert_eq!(
        cursor_rule.frontmatter.description,
        "React component guidelines"
    );
    assert!(!cursor_rule.frontmatter.always_apply);
    assert_eq!(cursor_rule.frontmatter.globs, "src/**/*.tsx, src/**/*.jsx");

    let agentsync_rule = cursor_rule_to_agentsync(&cursor_rule);

    // Verify inference: auto attached with globs → glob mode
    assert_eq!(
        agentsync_rule.frontmatter.globs,
        "src/**/*.tsx, src/**/*.jsx"
    );
    assert_eq!(agentsync_rule.frontmatter.targets, vec!["*"]);

    let cursor_cfg = agentsync_rule.frontmatter.cursor.as_ref().unwrap();
    assert!(!cursor_cfg.always_apply);
    assert_eq!(cursor_cfg.globs, "src/**/*.tsx, src/**/*.jsx");

    let windsurf_cfg = agentsync_rule.frontmatter.windsurf.as_ref().unwrap();
    assert_eq!(windsurf_cfg.trigger, WindsurfTrigger::Glob);
    assert_eq!(windsurf_cfg.globs, "src/**/*.tsx, src/**/*.jsx");

    let copilot_cfg = agentsync_rule.frontmatter.copilot.as_ref().unwrap();
    assert_eq!(copilot_cfg.apply_to, "src/**/*.tsx, src/**/*.jsx");

    // Verify content is preserved
    assert!(agentsync_rule.content.contains("React Components"));
    assert!(agentsync_rule.content.contains("functional components"));
}

#[test]
fn test_cursor_always_mode_conversion() {
    use indoc::indoc;

    let cursor_content = indoc! {r#"
        ---
        description: "Always applied rule"
        alwaysApply: true
        globs: ""
        ---

        # Always Rule

        This rule is always applied.
    "#};

    let cursor_rule: Rule<CursorRule> = parse_frontmatter(cursor_content, None).unwrap();
    let agentsync_rule = cursor_rule_to_agentsync(&cursor_rule);

    // Verify inference: always mode → always_on for all tools
    assert_eq!(agentsync_rule.frontmatter.globs, "**/*");

    let cursor_cfg = agentsync_rule.frontmatter.cursor.as_ref().unwrap();
    assert!(cursor_cfg.always_apply);

    let windsurf_cfg = agentsync_rule.frontmatter.windsurf.as_ref().unwrap();
    assert_eq!(windsurf_cfg.trigger, WindsurfTrigger::AlwaysOn);

    let copilot_cfg = agentsync_rule.frontmatter.copilot.as_ref().unwrap();
    assert_eq!(copilot_cfg.apply_to, "**");
}

// ============================================================================
// Windsurf Conversion Integration Tests
// ============================================================================

#[test]
fn test_windsurf_fixture_to_agentsync() {
    let windsurf_rule: Rule<WindsurfRule> =
        parse_frontmatter(WINDSURF_PYTHON_FIXTURE, None).unwrap();

    assert_eq!(
        windsurf_rule.frontmatter.trigger,
        WindsurfTrigger::ModelDecision
    );
    assert_eq!(
        windsurf_rule.frontmatter.description,
        "General python development rules"
    );
    assert_eq!(
        windsurf_rule.frontmatter.globs,
        "src/autopager/**/*.py, tests/**/*.py"
    );

    let agentsync_rule = windsurf_rule_to_agentsync(&windsurf_rule);

    // Verify inference: model_decision → auto attached without globs
    assert_eq!(agentsync_rule.frontmatter.globs, "**/*");

    let cursor_cfg = agentsync_rule.frontmatter.cursor.as_ref().unwrap();
    assert!(!cursor_cfg.always_apply);
    assert_eq!(cursor_cfg.globs, "");

    let windsurf_cfg = agentsync_rule.frontmatter.windsurf.as_ref().unwrap();
    assert_eq!(windsurf_cfg.trigger, WindsurfTrigger::ModelDecision);
    assert_eq!(windsurf_cfg.globs, "src/autopager/**/*.py, tests/**/*.py");

    // Verify content is preserved
    assert!(agentsync_rule.content.contains("Python Development"));
    assert!(agentsync_rule.content.contains("Clean Code"));
}

#[test]
fn test_windsurf_glob_mode_conversion() {
    use indoc::indoc;

    let windsurf_content = indoc! {r#"
        ---
        trigger: glob
        description: "Glob-based rule"
        globs: "**/*.ts, **/*.tsx"
        ---

        # TypeScript Rule

        TypeScript-specific guidelines.
    "#};

    let windsurf_rule: Rule<WindsurfRule> = parse_frontmatter(windsurf_content, None).unwrap();
    let agentsync_rule = windsurf_rule_to_agentsync(&windsurf_rule);

    // Verify inference: glob mode → auto attached with globs
    assert_eq!(agentsync_rule.frontmatter.globs, "**/*.ts, **/*.tsx");

    let cursor_cfg = agentsync_rule.frontmatter.cursor.as_ref().unwrap();
    assert!(!cursor_cfg.always_apply);
    assert_eq!(cursor_cfg.globs, "**/*.ts, **/*.tsx");

    let windsurf_cfg = agentsync_rule.frontmatter.windsurf.as_ref().unwrap();
    assert_eq!(windsurf_cfg.trigger, WindsurfTrigger::Glob);

    let copilot_cfg = agentsync_rule.frontmatter.copilot.as_ref().unwrap();
    assert_eq!(copilot_cfg.apply_to, "**/*.ts, **/*.tsx");
}

// ============================================================================
// Copilot Conversion Integration Tests
// ============================================================================

#[test]
fn test_copilot_fixture_to_agentsync() {
    let copilot_rule: Rule<CopilotRule> = parse_frontmatter(COPILOT_PYTHON_FIXTURE, None).unwrap();

    assert_eq!(
        copilot_rule.frontmatter.description,
        "Python development standards"
    );
    assert_eq!(copilot_rule.frontmatter.apply_to, "**/*.py");

    let agentsync_rule = copilot_rule_to_agentsync(&copilot_rule);

    // Verify inference: specific pattern → glob mode
    assert_eq!(agentsync_rule.frontmatter.globs, "**/*.py");

    let cursor_cfg = agentsync_rule.frontmatter.cursor.as_ref().unwrap();
    assert!(!cursor_cfg.always_apply);
    assert_eq!(cursor_cfg.globs, "**/*.py");

    let windsurf_cfg = agentsync_rule.frontmatter.windsurf.as_ref().unwrap();
    assert_eq!(windsurf_cfg.trigger, WindsurfTrigger::Glob);
    assert_eq!(windsurf_cfg.globs, "**/*.py");

    let copilot_cfg = agentsync_rule.frontmatter.copilot.as_ref().unwrap();
    assert_eq!(copilot_cfg.apply_to, "**/*.py");

    // Verify content is preserved
    assert!(agentsync_rule.content.contains("Python Standards"));
    assert!(agentsync_rule.content.contains("docstrings"));
}

#[test]
fn test_copilot_universal_pattern_conversion() {
    use indoc::indoc;

    let copilot_content = indoc! {r#"
        ---
        description: "Universal rule"
        applyTo: "**"
        ---

        # Universal Rule

        Applies to all files.
    "#};

    let copilot_rule: Rule<CopilotRule> = parse_frontmatter(copilot_content, None).unwrap();
    let agentsync_rule = copilot_rule_to_agentsync(&copilot_rule);

    // Verify inference: universal pattern → always mode
    assert_eq!(agentsync_rule.frontmatter.globs, "**/*");

    let cursor_cfg = agentsync_rule.frontmatter.cursor.as_ref().unwrap();
    assert!(cursor_cfg.always_apply);

    let windsurf_cfg = agentsync_rule.frontmatter.windsurf.as_ref().unwrap();
    assert_eq!(windsurf_cfg.trigger, WindsurfTrigger::AlwaysOn);

    let copilot_cfg = agentsync_rule.frontmatter.copilot.as_ref().unwrap();
    assert_eq!(copilot_cfg.apply_to, "**");
}

// ============================================================================
// AgentSync to Tool Conversion Tests
// ============================================================================

#[test]
fn test_agentsync_fixture_to_cursor() {
    let agentsync_rule: Rule<AgentSyncRule> =
        parse_frontmatter(AGENTSYNC_RUST_FIXTURE, None).unwrap();

    assert_eq!(
        agentsync_rule.frontmatter.description,
        "Comprehensive rule example"
    );
    assert_eq!(agentsync_rule.frontmatter.globs, "**/*.rs");
    assert_eq!(agentsync_rule.frontmatter.targets, vec!["*"]);

    let cursor_rule = agentsync_rule_to_cursor(&agentsync_rule);

    assert_eq!(
        cursor_rule.frontmatter.description,
        "Comprehensive rule example"
    );
    assert!(!cursor_rule.frontmatter.always_apply);
    assert_eq!(cursor_rule.frontmatter.globs, "**/*.rs");

    // Verify content is preserved
    assert!(cursor_rule.content.contains("Rust Development"));
    assert!(cursor_rule.content.contains("best practices"));
}

#[test]
fn test_agentsync_fixture_to_windsurf() {
    let agentsync_rule: Rule<AgentSyncRule> =
        parse_frontmatter(AGENTSYNC_RUST_FIXTURE, None).unwrap();

    let windsurf_rule = agentsync_rule_to_windsurf(&agentsync_rule);

    assert_eq!(
        windsurf_rule.frontmatter.description,
        "Comprehensive rule example"
    );
    assert_eq!(windsurf_rule.frontmatter.trigger, WindsurfTrigger::Glob);
    assert_eq!(windsurf_rule.frontmatter.globs, "**/*.rs");

    // Verify content is preserved
    assert!(windsurf_rule.content.contains("Rust Development"));
}

#[test]
fn test_agentsync_fixture_to_copilot() {
    let agentsync_rule: Rule<AgentSyncRule> =
        parse_frontmatter(AGENTSYNC_RUST_FIXTURE, None).unwrap();

    let copilot_rule = agentsync_rule_to_copilot(&agentsync_rule);

    assert_eq!(
        copilot_rule.frontmatter.description,
        "Comprehensive rule example"
    );
    assert_eq!(copilot_rule.frontmatter.apply_to, "**/*.rs");

    // Verify content is preserved
    assert!(copilot_rule.content.contains("Rust Development"));
}

// ============================================================================
// Roundtrip Tests
// ============================================================================

#[test]
fn test_roundtrip_cursor_fixture() {
    let original: Rule<CursorRule> = parse_frontmatter(CURSOR_REACT_FIXTURE, None).unwrap();
    let agentsync = cursor_rule_to_agentsync(&original);
    let back_to_cursor = agentsync_rule_to_cursor(&agentsync);

    assert_eq!(
        original.frontmatter.description,
        back_to_cursor.frontmatter.description
    );
    assert_eq!(
        original.frontmatter.always_apply,
        back_to_cursor.frontmatter.always_apply
    );
    assert_eq!(original.frontmatter.globs, back_to_cursor.frontmatter.globs);
    assert_eq!(original.content.trim(), back_to_cursor.content.trim());
}

#[test]
fn test_roundtrip_windsurf_fixture() {
    let original: Rule<WindsurfRule> = parse_frontmatter(WINDSURF_PYTHON_FIXTURE, None).unwrap();
    let agentsync = windsurf_rule_to_agentsync(&original);
    let back_to_windsurf = agentsync_rule_to_windsurf(&agentsync);

    assert_eq!(
        original.frontmatter.description,
        back_to_windsurf.frontmatter.description
    );
    assert_eq!(
        original.frontmatter.trigger,
        back_to_windsurf.frontmatter.trigger
    );
    assert_eq!(
        original.frontmatter.globs,
        back_to_windsurf.frontmatter.globs
    );
    assert_eq!(original.content.trim(), back_to_windsurf.content.trim());
}

#[test]
fn test_roundtrip_copilot_fixture() {
    let original: Rule<CopilotRule> = parse_frontmatter(COPILOT_PYTHON_FIXTURE, None).unwrap();
    let agentsync = copilot_rule_to_agentsync(&original);
    let back_to_copilot = agentsync_rule_to_copilot(&agentsync);

    assert_eq!(
        original.frontmatter.description,
        back_to_copilot.frontmatter.description
    );
    assert_eq!(
        original.frontmatter.apply_to,
        back_to_copilot.frontmatter.apply_to
    );
    assert_eq!(original.content.trim(), back_to_copilot.content.trim());
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_cursor_to_agentsync_serialization() {
    let cursor_rule: Rule<CursorRule> = parse_frontmatter(CURSOR_REACT_FIXTURE, None).unwrap();
    let agentsync_rule = cursor_rule_to_agentsync(&cursor_rule);

    // Serialize to string
    let serialized = serialize_frontmatter(&agentsync_rule).unwrap();

    // Parse it back
    let reparsed: Rule<AgentSyncRule> = parse_frontmatter(&serialized, None).unwrap();

    // Verify data is preserved
    assert_eq!(
        agentsync_rule.frontmatter.description,
        reparsed.frontmatter.description
    );
    assert_eq!(agentsync_rule.frontmatter.globs, reparsed.frontmatter.globs);
    assert_eq!(
        agentsync_rule.frontmatter.targets,
        reparsed.frontmatter.targets
    );
}

#[test]
fn test_agentsync_to_cursor_serialization() {
    let agentsync_rule: Rule<AgentSyncRule> =
        parse_frontmatter(AGENTSYNC_RUST_FIXTURE, None).unwrap();
    let cursor_rule = agentsync_rule_to_cursor(&agentsync_rule);

    // Serialize to string
    let serialized = serialize_frontmatter(&cursor_rule).unwrap();

    // Parse it back
    let reparsed: Rule<CursorRule> = parse_frontmatter(&serialized, None).unwrap();

    // Verify data is preserved
    assert_eq!(
        cursor_rule.frontmatter.description,
        reparsed.frontmatter.description
    );
    assert_eq!(
        cursor_rule.frontmatter.always_apply,
        reparsed.frontmatter.always_apply
    );
    assert_eq!(cursor_rule.frontmatter.globs, reparsed.frontmatter.globs);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_description_handling() {
    let cursor = CursorRule {
        description: String::new(),
        always_apply: true,
        globs: String::new(),
    };

    let agentsync = agentsync::converter::cursor_to_agentsync(&cursor);
    assert_eq!(agentsync.description, "");

    let back_to_cursor = agentsync::converter::agentsync_to_cursor(&agentsync);
    assert_eq!(back_to_cursor.description, "");
}

#[test]
fn test_complex_glob_patterns() {
    let complex_globs = "src/**/*.{ts,tsx}, tests/**/*.test.ts, !**/*.spec.ts";

    let cursor = CursorRule {
        description: "Complex globs".to_string(),
        always_apply: false,
        globs: complex_globs.to_string(),
    };

    let agentsync = agentsync::converter::cursor_to_agentsync(&cursor);
    let back_to_cursor = agentsync::converter::agentsync_to_cursor(&agentsync);

    // Verify globs are normalized (spaces added around commas) but preserved
    let normalized = "src/**/*.{ts, tsx}, tests/**/*.test.ts, !**/*.spec.ts";
    assert_eq!(back_to_cursor.globs, normalized);
}

#[test]
fn test_missing_tool_configs_use_fallback() {
    let agentsync = AgentSyncRule {
        targets: vec!["*".to_string()],
        description: "No tool configs".to_string(),
        globs: "**/*.py".to_string(),
        cursor: None,
        windsurf: None,
        copilot: None,
    };

    // Should use fallback logic based on global globs
    let cursor = agentsync::converter::agentsync_to_cursor(&agentsync);
    assert_eq!(cursor.globs, "**/*.py");

    let windsurf = agentsync::converter::agentsync_to_windsurf(&agentsync);
    assert_eq!(windsurf.globs, "**/*.py");

    let copilot = agentsync::converter::agentsync_to_copilot(&agentsync);
    assert_eq!(copilot.apply_to, "**/*.py");
}
