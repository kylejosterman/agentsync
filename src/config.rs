//! Load, validate, and save agentsync.json configuration.

use crate::fs::write_atomic;
use crate::models::AgentSyncConfig;
use crate::{AgentSyncError, Result};
use fs_err as fs;
use std::path::Path;

/// Load configuration from agentsync.json
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AgentSyncConfig> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(AgentSyncError::ConfigNotFound {
            path: path.display().to_string(),
        });
    }

    let contents = fs::read_to_string(path)?;

    let config: AgentSyncConfig =
        serde_json::from_str(&contents).map_err(AgentSyncError::JsonParse)?;

    // Validate the configuration
    config.validate()?;

    Ok(config)
}

/// Save config atomically
pub fn save_config<P: AsRef<Path>>(path: P, config: &AgentSyncConfig) -> Result<()> {
    let path = path.as_ref();

    // Validate before saving
    config.validate()?;

    let json = serde_json::to_string_pretty(config).map_err(AgentSyncError::JsonParse)?;

    // Use atomic write to prevent corruption
    write_atomic(path, json)?;

    Ok(())
}

/// Create default config
pub fn create_default_config() -> AgentSyncConfig {
    AgentSyncConfig::default()
}

#[cfg(test)]
mod tests {
    // Allow expect/unwrap in tests for brevity
    #![allow(clippy::expect_used)]
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() {
        use indoc::indoc;

        let mut file = NamedTempFile::new().expect("should create temp file");
        writeln!(
            file,
            indoc! {r#"
                {{
                  "tools": ["cursor", "copilot", "windsurf"],
                  "baseDirs": ["."]
                }}
            "#}
        )
        .expect("test operation should succeed");

        let config = load_config(file.path()).expect("should load config");
        assert_eq!(config.tools.len(), 3);
        assert_eq!(config.base_dirs, vec!["."]);
    }

    #[test]
    fn test_load_config_with_subset_tools() {
        use indoc::indoc;

        let mut file = NamedTempFile::new().expect("should create temp file");
        writeln!(
            file,
            indoc! {r#"
                {{
                  "tools": ["cursor"],
                  "baseDirs": ["."]
                }}
            "#}
        )
        .expect("test operation should succeed");

        let config = load_config(file.path()).expect("should load config");
        assert_eq!(config.tools, vec!["cursor"]);
    }

    #[test]
    fn test_load_config_monorepo() {
        use indoc::indoc;

        let mut file = NamedTempFile::new().expect("should create temp file");
        writeln!(
            file,
            indoc! {r#"
                {{
                  "tools": ["cursor", "windsurf"],
                  "baseDirs": [".", "packages/frontend", "packages/backend"]
                }}
            "#}
        )
        .expect("test operation should succeed");

        let config = load_config(file.path()).expect("should load config");
        assert_eq!(config.tools.len(), 2);
        assert_eq!(config.base_dirs.len(), 3);
        assert_eq!(config.base_dirs[1], "packages/frontend");
    }

    #[test]
    fn test_load_invalid_tool() {
        use indoc::indoc;

        let mut file = NamedTempFile::new().expect("should create temp file");
        writeln!(
            file,
            indoc! {r#"
                {{
                  "tools": ["invalid_tool"],
                  "baseDirs": ["."]
                }}
            "#}
        )
        .expect("test operation should succeed");

        let result = load_config(file.path());
        assert!(result.is_err());
        let err = result.expect_err("should be an error");
        match err {
            AgentSyncError::ConfigError { error: msg } => {
                assert!(msg.contains("invalid_tool"));
                assert!(msg.contains("Valid tools"));
            }
            _ => unreachable!("Expected ConfigError, got: {err:?}"),
        }
    }

    #[test]
    fn test_load_empty_base_dirs() {
        use indoc::indoc;

        let mut file = NamedTempFile::new().expect("should create temp file");
        writeln!(
            file,
            indoc! {r#"
                {{
                  "tools": ["cursor"],
                  "baseDirs": []
                }}
            "#}
        )
        .expect("test operation should succeed");

        let result = load_config(file.path());
        assert!(result.is_err());
        let err = result.expect_err("should be an error");
        match err {
            AgentSyncError::ConfigError { error: msg } => {
                assert!(msg.contains("baseDirs"));
            }
            _ => unreachable!("Expected ConfigError, got: {err:?}"),
        }
    }

    #[test]
    fn test_load_malformed_json() {
        let mut file = NamedTempFile::new().expect("should create temp file");
        writeln!(file, "{{ invalid json }}").expect("should write");

        let result = load_config(file.path());
        assert!(result.is_err());
        let err = result.expect_err("should be an error");
        match err {
            AgentSyncError::JsonParse(_) => {}
            _ => unreachable!("Expected JsonParse error, got: {err:?}"),
        }
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_config("/nonexistent/agentsync.json");
        assert!(result.is_err());
        let err = result.expect_err("should be an error");
        match err {
            AgentSyncError::ConfigNotFound { .. } => {}
            _ => unreachable!("Expected ConfigNotFound error, got: {err:?}"),
        }
    }

    #[test]
    fn test_save_and_load_config() {
        let file = NamedTempFile::new().expect("should create temp file");

        let config = AgentSyncConfig {
            tools: vec!["cursor".to_string(), "windsurf".to_string()],
            base_dirs: vec![".".to_string()],
        };

        save_config(file.path(), &config).expect("should save config");
        let loaded_config = load_config(file.path()).expect("should load config");

        assert_eq!(config.tools, loaded_config.tools);
        assert_eq!(config.base_dirs, loaded_config.base_dirs);
    }

    #[test]
    fn test_save_invalid_config() {
        let file = NamedTempFile::new().expect("should create temp file");

        let config = AgentSyncConfig {
            tools: vec!["invalid".to_string()],
            base_dirs: vec![".".to_string()],
        };

        let result = save_config(file.path(), &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_default_config() {
        let config = create_default_config();
        assert_eq!(config.tools.len(), 3);
        assert!(config.tools.contains(&"cursor".to_string()));
        assert!(config.tools.contains(&"copilot".to_string()));
        assert!(config.tools.contains(&"windsurf".to_string()));
        assert_eq!(config.base_dirs, vec!["."]);
    }

    #[test]
    fn test_config_validation() {
        let valid_config = AgentSyncConfig {
            tools: vec!["cursor".to_string()],
            base_dirs: vec![".".to_string()],
        };
        assert!(valid_config.validate().is_ok());

        let invalid_tool_config = AgentSyncConfig {
            tools: vec!["unknown".to_string()],
            base_dirs: vec![".".to_string()],
        };
        assert!(invalid_tool_config.validate().is_err());

        let empty_dirs_config = AgentSyncConfig {
            tools: vec!["cursor".to_string()],
            base_dirs: vec![],
        };
        assert!(empty_dirs_config.validate().is_err());
    }

    #[test]
    fn test_config_validation_with_suggestions() {
        // Test typo suggestions
        let typo_config = AgentSyncConfig {
            tools: vec!["github-copilot".to_string()],
            base_dirs: vec![".".to_string()],
        };
        let result = typo_config.validate();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Did you mean 'copilot'?"));

        // Test unsupported tool
        let unsupported_config = AgentSyncConfig {
            tools: vec!["codeium".to_string()],
            base_dirs: vec![".".to_string()],
        };
        let result = unsupported_config.validate();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not yet supported"));
    }

    #[test]
    fn test_config_validation_path_traversal() {
        // Test path traversal in baseDirs
        let traversal_config = AgentSyncConfig {
            tools: vec!["cursor".to_string()],
            base_dirs: vec![".".to_string(), "../other-project".to_string()],
        };
        let result = traversal_config.validate();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("path traversal") || err_msg.contains("Invalid baseDir"));
    }

    #[test]
    fn test_config_validation_empty_base_dir() {
        let empty_base_dir_config = AgentSyncConfig {
            tools: vec!["cursor".to_string()],
            base_dirs: vec![".".to_string(), String::new()],
        };
        let result = empty_base_dir_config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_validation_all_valid_tools() {
        let all_tools_config = AgentSyncConfig {
            tools: vec![
                "cursor".to_string(),
                "copilot".to_string(),
                "windsurf".to_string(),
            ],
            base_dirs: vec![".".to_string()],
        };
        assert!(all_tools_config.validate().is_ok());
    }
}
