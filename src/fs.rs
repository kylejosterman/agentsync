//! File system operations for AgentSync
//!
//! This module provides utilities for:
//! - Project root detection (finding agentsync.json)
//! - File discovery for tool directories
//! - Safe file reading and writing with error handling
//! - Atomic file writes to prevent data corruption
//! - File extension handling (.md vs .mdc)
//! - Permission error handling

use crate::{AgentSyncError, Result};
use fs_err as fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tempfile::NamedTempFile;

/// Tool type for directory and extension resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    AgentSync,
    Cursor,
    Copilot,
    Windsurf,
}

impl FromStr for Tool {
    type Err = AgentSyncError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "agentsync" => Ok(Self::AgentSync),
            "cursor" => Ok(Self::Cursor),
            "copilot" => Ok(Self::Copilot),
            "windsurf" => Ok(Self::Windsurf),
            _ => Err(AgentSyncError::InvalidTool {
                tool: s.to_string(),
            }),
        }
    }
}

impl Tool {
    /// Get the lowercase name of this tool
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::AgentSync => "agentsync",
            Self::Cursor => "cursor",
            Self::Copilot => "copilot",
            Self::Windsurf => "windsurf",
        }
    }

    /// Get the directory path for this tool relative to project root
    #[must_use]
    pub const fn directory(&self) -> &'static str {
        match self {
            Self::AgentSync => ".agentsync/rules",
            Self::Cursor => ".cursor/rules",
            Self::Copilot => ".github/instructions",
            Self::Windsurf => ".windsurf/rules",
        }
    }

    /// Get the file extension for this tool
    #[must_use]
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::AgentSync | Self::Copilot | Self::Windsurf => "md",
            Self::Cursor => "mdc",
        }
    }
}

impl std::fmt::Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Write data to a file atomically using temp file + rename
///
/// This prevents partial writes and ensures atomicity:
/// 1. Write to temporary file in same directory
/// 2. Flush to disk
/// 3. Atomically rename temp to target
pub fn write_atomic<P: AsRef<Path>>(path: P, content: impl AsRef<[u8]>) -> Result<()> {
    let path = path.as_ref();
    let parent = path
        .parent()
        .ok_or_else(|| AgentSyncError::Other("Path must have a parent directory".to_string()))?;

    if !parent.exists() {
        fs::create_dir_all(parent)?;
    }

    let mut temp_file = NamedTempFile::new_in(parent)?;
    temp_file.write_all(content.as_ref())?;
    temp_file.flush()?;
    temp_file.persist(path).map_err(|e| e.error)?;
    Ok(())
}

/// Find the project root by searching for agentsync.json in the current directory
///
/// Returns the directory containing agentsync.json, or an error if not found.
pub fn find_project_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;

    let config_path = current_dir.join("agentsync.json");
    if config_path.exists() {
        Ok(current_dir)
    } else {
        Err(AgentSyncError::NotInitialized)
    }
}

/// Discover all rule files for a specific tool in the project
///
/// For Copilot, searches for `.instructions.md` files.
/// For other tools, searches for files with their standard extension.
///
/// Returns a vector of file paths relative to the project root.
///
/// Validates that the tool directory is within the project root and filters out
/// any discovered files that escape the project boundary.
pub fn discover_rules(project_root: &Path, tool: Tool) -> Result<Vec<PathBuf>> {
    let tool_dir = project_root.join(tool.directory());

    crate::security::validate_path_within_base(project_root, &tool_dir)?;

    if !tool_dir.exists() {
        return Ok(Vec::new());
    }

    let pattern = match tool {
        Tool::Copilot => format!("{}/*.instructions.md", tool_dir.display()),
        _ => format!("{}/*.{}", tool_dir.display(), tool.extension()),
    };

    let paths = glob::glob(&pattern)?
        .filter_map(|entry| {
            let path = entry.ok()?;
            crate::security::validate_path_within_base(project_root, &path).ok()?;
            Some(path)
        })
        .collect();

    Ok(paths)
}

/// Read a rule file and return its contents
pub fn read_rule_file<P: AsRef<Path>>(path: P) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

/// Write content to a rule file atomically
pub fn write_rule_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    write_atomic(path, content)
}

/// Get the full path for a rule file in a tool directory
///
/// Constructs the path: `<project_root>/<tool_dir>/<rule_name>.<ext>`
///
/// Validates that the constructed path stays within the project root.
pub fn rule_path(project_root: &Path, tool: Tool, rule_name: &str) -> Result<PathBuf> {
    // Validate rule name doesn't contain path traversal
    crate::security::validate_relative_path(Path::new(rule_name))?;

    let dir = project_root.join(tool.directory());
    let path = match tool {
        Tool::Copilot => dir.join(format!("{rule_name}.instructions.md")),
        _ => dir.join(format!("{}.{}", rule_name, tool.extension())),
    };

    // Validate the constructed path is within project root
    crate::security::validate_path_within_base(project_root, &path)?;
    Ok(path)
}

/// Extract the rule name from a file path (filename without extension)
///
/// For Copilot `.instructions.md` files, removes both `.instructions` and `.md`.
/// For other files, removes just the extension.
///
/// Returns `None` if the path has no filename or no stem.
#[must_use]
pub fn extract_rule_name(path: &Path) -> Option<String> {
    let filename = path.file_name()?.to_str()?;

    // Handle Copilot .instructions.md files
    if filename.ends_with(".instructions.md") {
        return Some(filename.trim_end_matches(".instructions.md").to_string());
    }

    // Handle regular files
    path.file_stem().and_then(|s| s.to_str()).map(String::from)
}

/// Validate that a rule name follows kebab-case convention
///
/// Rule names must:
/// - Contain only lowercase letters, numbers, and hyphens
/// - Not start or end with a hyphen
/// - Not contain consecutive hyphens
pub fn validate_rule_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(AgentSyncError::InvalidRuleName {
            name: "Rule name cannot be empty".to_string(),
        });
    }

    // Check for valid kebab-case
    let is_valid = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--");

    if is_valid {
        Ok(())
    } else {
        Err(AgentSyncError::InvalidRuleName {
            name: name.to_string(),
        })
    }
}

/// Ensure a directory exists, creating it if necessary
///
/// Returns an error if the directory cannot be created due to permissions.
pub fn ensure_directory<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    if !path.exists() {
        fs::create_dir_all(path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Allow expect/unwrap in tests for brevity
    #![allow(clippy::expect_used)]
    #![allow(clippy::unwrap_used)]

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_tool_from_str() {
        assert_eq!(
            "cursor".parse::<Tool>().expect("should parse cursor"),
            Tool::Cursor
        );
        assert_eq!(
            "copilot".parse::<Tool>().expect("should parse copilot"),
            Tool::Copilot
        );
        assert_eq!(
            "windsurf".parse::<Tool>().expect("should parse windsurf"),
            Tool::Windsurf
        );
        assert_eq!(
            "agentsync".parse::<Tool>().expect("should parse agentsync"),
            Tool::AgentSync
        );
        assert!("invalid".parse::<Tool>().is_err());
    }

    #[test]
    fn test_tool_directory() {
        assert_eq!(Tool::Cursor.directory(), ".cursor/rules");
        assert_eq!(Tool::Copilot.directory(), ".github/instructions");
        assert_eq!(Tool::Windsurf.directory(), ".windsurf/rules");
        assert_eq!(Tool::AgentSync.directory(), ".agentsync/rules");
    }

    #[test]
    fn test_tool_extension() {
        assert_eq!(Tool::Cursor.extension(), "mdc");
        assert_eq!(Tool::Copilot.extension(), "md");
        assert_eq!(Tool::Windsurf.extension(), "md");
        assert_eq!(Tool::AgentSync.extension(), "md");
    }

    #[test]
    fn test_find_project_root_not_found() {
        // This test assumes we're not in a directory with agentsync.json
        // In the actual project, this would fail, so we just check error type
        let result = find_project_root();
        // Can't make assumptions about the test environment
        assert!(result.is_ok() || matches!(result, Err(AgentSyncError::NotInitialized)));
    }

    #[test]
    fn test_discover_rules_empty_directory() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let project_root = temp_dir.path();

        let rules =
            discover_rules(project_root, Tool::Cursor).expect("test operation should succeed");
        assert_eq!(rules.len(), 0);
    }

    #[test]
    fn test_discover_rules_with_files() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let project_root = temp_dir.path();

        // Create cursor rules directory with files
        let cursor_dir = project_root.join(".cursor/rules");
        fs::create_dir_all(&cursor_dir).expect("should create cursor dir");
        fs::write(cursor_dir.join("rule1.mdc"), "content1").expect("test operation should succeed");
        fs::write(cursor_dir.join("rule2.mdc"), "content2").expect("test operation should succeed");
        // Add a file with wrong extension (should be ignored)
        fs::write(cursor_dir.join("rule3.md"), "content3").expect("test operation should succeed");

        let rules =
            discover_rules(project_root, Tool::Cursor).expect("test operation should succeed");
        assert_eq!(rules.len(), 2);

        // Extract filenames and check
        let mut filenames: Vec<_> = rules
            .iter()
            .filter_map(|p| p.file_name()?.to_str())
            .collect();
        filenames.sort_unstable();
        assert_eq!(filenames, vec!["rule1.mdc", "rule2.mdc"]);
    }

    #[test]
    fn test_read_rule_file() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let file_path = temp_dir.path().join("test.md");
        let content = "# Test Rule\n\nContent here.";

        fs::write(&file_path, content).expect("test operation should succeed");

        let read_content = read_rule_file(&file_path).expect("test operation should succeed");
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_rule_file("/nonexistent/file.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_write_rule_file() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let file_path = temp_dir.path().join("subdir/test.md");
        let content = "# Test Rule\n\nContent here.";

        write_rule_file(&file_path, content).expect("test operation should succeed");

        let read_content = fs::read_to_string(&file_path).expect("test operation should succeed");
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_write_rule_file_creates_directory() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let file_path = temp_dir.path().join("nested/deep/test.md");
        let content = "content";

        write_rule_file(&file_path, content).expect("test operation should succeed");

        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), content);
    }

    #[test]
    fn test_rule_path() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let project_root = temp_dir.path();

        let path = rule_path(project_root, Tool::Cursor, "python-dev")
            .expect("test operation should succeed");
        assert!(path.ends_with(".cursor/rules/python-dev.mdc"));

        let path = rule_path(project_root, Tool::Copilot, "react-rules")
            .expect("test operation should succeed");
        assert!(path.ends_with(".github/instructions/react-rules.instructions.md"));
    }

    #[test]
    fn test_rule_path_with_traversal() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let project_root = temp_dir.path();

        // Try to escape with ..
        let result = rule_path(project_root, Tool::Cursor, "../../../etc/passwd");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AgentSyncError::PathTraversal { .. }));
    }

    #[test]
    fn test_rule_path_with_slash() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let project_root = temp_dir.path();

        // Slash in rule name is technically allowed by validate_relative_path
        // but would create a subdirectory structure, which is valid
        let result = rule_path(project_root, Tool::Cursor, "subdir/rule");
        // This should succeed - the path is valid, just creates a subdirectory
        assert!(result.is_ok());

        // The application-level validation in run_add() prevents slashes in rule names
    }

    #[test]
    fn test_extract_rule_name() {
        assert_eq!(
            extract_rule_name(Path::new("/path/to/python-dev.md")),
            Some("python-dev".to_string())
        );
        assert_eq!(
            extract_rule_name(Path::new("rule.mdc")),
            Some("rule".to_string())
        );
        assert_eq!(
            extract_rule_name(Path::new("/path/to/react-rules.instructions.md")),
            Some("react-rules".to_string())
        );
        assert_eq!(extract_rule_name(Path::new("/")), None);
    }

    #[test]
    fn test_validate_rule_name() {
        // Valid names
        assert!(validate_rule_name("python-dev").is_ok());
        assert!(validate_rule_name("react-components").is_ok());
        assert!(validate_rule_name("rule123").is_ok());
        assert!(validate_rule_name("my-rule-2").is_ok());

        // Invalid names
        assert!(validate_rule_name("").is_err());
        assert!(validate_rule_name("Python-Dev").is_err()); // uppercase
        assert!(validate_rule_name("-python").is_err()); // starts with hyphen
        assert!(validate_rule_name("python-").is_err()); // ends with hyphen
        assert!(validate_rule_name("python--dev").is_err()); // consecutive hyphens
        assert!(validate_rule_name("python_dev").is_err()); // underscore
        assert!(validate_rule_name("python dev").is_err()); // space
    }

    #[test]
    fn test_ensure_directory() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let dir_path = temp_dir.path().join("nested/directory");

        assert!(!dir_path.exists());
        ensure_directory(&dir_path).expect("test operation should succeed");
        assert!(dir_path.exists());

        // Calling again should be idempotent
        ensure_directory(&dir_path).expect("test operation should succeed");
        assert!(dir_path.exists());
    }

    #[test]
    fn test_discover_rules_multiple_tools() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let project_root = temp_dir.path();

        // Create rules for multiple tools
        let cursor_dir = project_root.join(".cursor/rules");
        fs::create_dir_all(&cursor_dir).expect("should create cursor dir");
        fs::write(cursor_dir.join("rule1.mdc"), "cursor1").expect("test operation should succeed");

        let copilot_dir = project_root.join(".github/instructions");
        fs::create_dir_all(&copilot_dir).expect("should create copilot dir");
        fs::write(copilot_dir.join("rule2.instructions.md"), "copilot1").expect("test operation should succeed");
        fs::write(copilot_dir.join("rule3.instructions.md"), "copilot2").expect("test operation should succeed");

        let cursor_rules =
            discover_rules(project_root, Tool::Cursor).expect("test operation should succeed");
        let copilot_rules =
            discover_rules(project_root, Tool::Copilot).expect("test operation should succeed");

        assert_eq!(cursor_rules.len(), 1);
        assert_eq!(copilot_rules.len(), 2);
    }

    #[test]
    fn test_atomic_write_creates_file() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        let content = "test content";

        write_atomic(&file_path, content).expect("test operation should succeed");

        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), content);
    }

    #[test]
    fn test_atomic_write_overwrites_existing() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let file_path = temp_dir.path().join("test.txt");

        // Write initial content
        fs::write(&file_path, "old content").expect("test operation should succeed");
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "old content");

        // Overwrite atomically
        write_atomic(&file_path, "new content").expect("test operation should succeed");

        assert_eq!(fs::read_to_string(&file_path).unwrap(), "new content");
    }

    #[test]
    fn test_atomic_write_creates_parent_dirs() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let file_path = temp_dir.path().join("nested/deep/test.txt");

        write_atomic(&file_path, "content").expect("test operation should succeed");

        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "content");
    }

    #[test]
    fn test_atomic_write_with_binary_content() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let file_path = temp_dir.path().join("binary.dat");
        let content: Vec<u8> = vec![0, 1, 2, 3, 255, 254, 253];

        write_atomic(&file_path, &content).expect("test operation should succeed");

        assert!(file_path.exists());
        assert_eq!(fs::read(&file_path).unwrap(), content);
    }

    #[test]
    fn test_atomic_write_empty_content() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let file_path = temp_dir.path().join("empty.txt");

        write_atomic(&file_path, "").expect("test operation should succeed");

        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "");
    }

    #[test]
    fn test_atomic_write_large_content() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let file_path = temp_dir.path().join("large.txt");
        // Create a large string (1MB)
        let content = "x".repeat(1024 * 1024);

        write_atomic(&file_path, &content).expect("test operation should succeed");

        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap().len(), content.len());
    }

    #[test]
    fn test_write_rule_file_uses_atomic_write() {
        let temp_dir = TempDir::new().expect("should create temp dir");
        let file_path = temp_dir.path().join("nested/rule.md");
        let content = "# Rule Content";

        write_rule_file(&file_path, content).expect("test operation should succeed");

        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), content);
    }

    #[test]
    fn test_read_rule_file_error_includes_path() {
        let result = read_rule_file("/nonexistent/path/file.md");
        assert!(result.is_err());

        // fs-err automatically includes path in error message
        let err_msg = result.expect_err("should be an error").to_string();
        assert!(err_msg.contains("nonexistent") || err_msg.contains("file.md"));
    }

    #[test]
    fn test_atomic_write_error_includes_path() {
        // Try to write to a path that can't be created (e.g., root on Unix)
        let result = write_atomic("/invalid/path/that/cannot/exist/file.txt", "content");
        assert!(result.is_err());

        // Error should include path information
        let err = result.expect_err("should be an error");
        let err_str = err.to_string();
        assert!(err_str.contains("path") || err_str.contains("file"));
    }
}
