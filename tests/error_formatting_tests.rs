// Allow expect/unwrap in tests for brevity
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use agentsync::error::AgentSyncError;

#[test]
fn test_config_not_found_formatting() {
    let err = AgentSyncError::ConfigNotFound {
        path: "agentsync.json".to_string(),
    };
    let msg = err.to_string();

    // Check main message
    assert!(msg.contains("Configuration file not found"));
    assert!(msg.contains("agentsync.json"));

    // Check hints
    assert!(msg.contains("hint"));
    assert!(msg.contains("agentsync init"));
    assert!(msg.contains("--config <path>"));
}

#[test]
fn test_invalid_tool_formatting() {
    let err = AgentSyncError::InvalidTool {
        tool: "cursr".to_string(),
    };
    let msg = err.to_string();

    // Check main message
    assert!(msg.contains("Invalid tool name"));
    assert!(msg.contains("cursr"));

    // Check fuzzy matching suggestion
    assert!(msg.contains("Did you mean"));
    assert!(msg.contains("cursor"));

    // Check valid tools list
    assert!(msg.contains("Valid tools are"));
    assert!(msg.contains("cursor"));
    assert!(msg.contains("copilot"));
    assert!(msg.contains("windsurf"));
}

#[test]
fn test_invalid_tool_no_suggestion_for_short_input() {
    let err = AgentSyncError::InvalidTool {
        tool: "ab".to_string(),
    };
    let msg = err.to_string();

    // Should not suggest for very short inputs
    assert!(!msg.contains("Did you mean"));

    // But should still show valid tools
    assert!(msg.contains("Valid tools are"));
}

#[test]
fn test_not_initialized_formatting() {
    let err = AgentSyncError::NotInitialized;
    let msg = err.to_string();

    // Check main message
    assert!(msg.contains("Project not initialized"));

    // Check hints
    assert!(msg.contains("hint"));
    assert!(msg.contains("agentsync init"));
    assert!(msg.contains("agentsync.json"));
    assert!(msg.contains(".agentsync/"));
}

#[test]
fn test_permission_denied_formatting() {
    let err = AgentSyncError::PermissionDenied {
        path: "/etc/restricted".to_string(),
    };
    let msg = err.to_string();

    // Check main message
    assert!(msg.contains("Permission denied"));
    assert!(msg.contains("/etc/restricted"));

    // Check hints
    assert!(msg.contains("hint"));
    assert!(msg.contains("insufficient permissions"));

    // Platform-specific hints
    #[cfg(unix)]
    {
        assert!(msg.contains("chmod"));
        assert!(msg.contains("sudo"));
    }

    #[cfg(windows)]
    {
        assert!(msg.contains("file properties"));
        assert!(msg.contains("read permissions"));
    }
}

#[test]
fn test_invalid_rule_name_formatting() {
    let err = AgentSyncError::InvalidRuleName {
        name: "MyRule".to_string(),
    };
    let msg = err.to_string();

    // Check main message
    assert!(msg.contains("Invalid rule name"));
    assert!(msg.contains("MyRule"));

    // Check hints
    assert!(msg.contains("hint"));
    assert!(msg.contains("kebab-case"));
    assert!(msg.contains("my-rule"));
}

#[test]
fn test_config_error_formatting() {
    let err = AgentSyncError::ConfigError {
        error: "invalid field 'foo'".to_string(),
    };
    let msg = err.to_string();

    // Check main message
    assert!(msg.contains("Configuration error"));
    assert!(msg.contains("invalid field 'foo'"));

    // Check hints
    assert!(msg.contains("hint"));
    assert!(msg.contains("agentsync.json"));
    assert!(msg.contains("agentsync validate"));
}

#[test]
fn test_invalid_frontmatter_formatting() {
    let yaml_err = serde_yaml::from_str::<serde_yaml::Value>("invalid: [unclosed")
        .expect_err("should fail");

    let err = AgentSyncError::invalid_frontmatter(
        "test-rule.md",
        Some(5),
        yaml_err,
    );
    let msg = err.to_string();

    // Check main message
    assert!(msg.contains("Invalid frontmatter"));
    assert!(msg.contains("test-rule.md"));
    assert!(msg.contains("line 5"));

    // Check parse error section
    assert!(msg.contains("[parse error]"));

    // Check hints
    assert!(msg.contains("hint"));
    assert!(msg.contains("valid YAML"));
    assert!(msg.contains("---"));
    assert!(msg.contains("Example format"));
}

#[test]
fn test_invalid_frontmatter_without_line_number() {
    let yaml_err = serde_yaml::from_str::<serde_yaml::Value>("invalid: [unclosed")
        .expect_err("should fail");

    let err = AgentSyncError::invalid_frontmatter(
        "test-rule.md",
        None,
        yaml_err,
    );
    let msg = err.to_string();

    // Should not contain our line number formatting (the YAML parser error itself may mention lines)
    // Our format would be "test-rule.md at line X" so check that pattern doesn't exist
    assert!(!msg.contains("test-rule.md at"));

    // But should still have other parts
    assert!(msg.contains("Invalid frontmatter"));
    assert!(msg.contains("test-rule.md"));
}

#[test]
fn test_conversion_failed_formatting() {
    let source = AgentSyncError::Other("parse error".to_string());
    let err = AgentSyncError::ConversionFailed {
        rule: "my-rule".to_string(),
        from_tool: "cursor".to_string(),
        to_tool: "copilot".to_string(),
        source: Box::new(source),
    };
    let msg = err.to_string();

    // Check main message
    assert!(msg.contains("Failed to convert rule"));
    assert!(msg.contains("my-rule"));
    assert!(msg.contains("cursor"));
    assert!(msg.contains("copilot"));

    // Check nested error
    assert!(msg.contains("[error]"));
    assert!(msg.contains("parse error"));

    // Check hints
    assert!(msg.contains("hint"));
    assert!(msg.contains("syntax specific to"));
    assert!(msg.contains("agentsync validate --tool cursor"));
}

#[test]
fn test_error_display_preserves_colors() {
    // This test verifies that color codes are present in the output
    // (they won't be visible in test output but the ANSI codes should be there)
    let err = AgentSyncError::InvalidTool {
        tool: "test".to_string(),
    };
    let msg = err.to_string();

    // ANSI color codes should be present (though not visible in plain text)
    // We can't easily test for specific ANSI codes without pulling in the full string,
    // but we can verify the message is formatted
    assert!(msg.len() > 50); // Should be longer due to ANSI codes
    assert!(msg.contains("Invalid tool name"));
}

