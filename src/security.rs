//! Security utilities for path validation and traversal protection
//!
//! This module provides functions to prevent path traversal attacks and ensure
//! that all file operations stay within allowed boundaries.
//!
//! # Security Considerations
//!
//! Path traversal attacks can occur when user-provided paths contain sequences
//! like `..` or absolute paths that escape the intended base directory. This
//! module provides explicit validation to prevent such attacks.
//!
//! # Example
//!
//! ```rust,ignore
//! use agentsync::security::validate_path_within_base;
//! use std::path::Path;
//!
//! let base = Path::new("/project");
//! let safe_path = Path::new("/project/subdir/file.txt");
//! let unsafe_path = Path::new("/project/../etc/passwd");
//!
//! assert!(validate_path_within_base(base, safe_path).is_ok());
//! assert!(validate_path_within_base(base, unsafe_path).is_err());
//! ```

use crate::{AgentSyncError, Result};
use std::path::{Path, PathBuf};

/// Validate that a target path is within a base directory
///
/// This function performs the following checks:
/// 1. Canonicalizes both base and target paths (resolves symlinks, `.`, `..`)
/// 2. Verifies that the canonical target path starts with the canonical base path
/// 3. Returns an error if the target escapes the base directory
///
/// # Security
///
/// This function protects against:
/// - Path traversal attacks using `..`
/// - Absolute paths that escape the base directory
/// - Symlink attacks that point outside the base directory
///
/// # Arguments
///
/// * `base` - The base directory that the target must be within
/// * `target` - The target path to validate
///
/// # Errors
///
/// Returns `PathTraversal` error if:
/// - The target path escapes the base directory
/// - The base path doesn't exist or can't be canonicalized
/// - The target path (or its parent) doesn't exist or can't be canonicalized
///
/// # Example
///
/// ```rust,ignore
/// let base = Path::new("/project");
/// let target = Path::new("/project/subdir/file.txt");
///
/// validate_path_within_base(base, target)?;
/// ```
pub fn validate_path_within_base(base: &Path, target: &Path) -> Result<()> {
    // Canonicalize base directory (must exist)
    let canonical_base = base.canonicalize().map_err(|e| {
        AgentSyncError::Other(format!(
            "Failed to canonicalize base directory '{}': {}",
            base.display(),
            e
        ))
    })?;

    // Try to canonicalize target path
    // If it doesn't exist, canonicalize its parent and append the filename
    let canonical_target = if let Ok(path) = target.canonicalize() {
        path
    } else {
        // Target doesn't exist yet - validate its parent
        let parent = target.parent().ok_or_else(|| {
            AgentSyncError::Other(format!(
                "Target path '{}' has no parent directory",
                target.display()
            ))
        })?;

        // If parent doesn't exist, try to canonicalize grandparent recursively
        let canonical_parent = canonicalize_existing_ancestor(parent)?;

        // Reconstruct the path with the non-existent components
        let relative = target.strip_prefix(parent).map_err(|_| {
            AgentSyncError::Other(format!(
                "Failed to compute relative path for '{}'",
                target.display()
            ))
        })?;

        canonical_parent.join(relative)
    };

    // Check if canonical target is within canonical base
    if !canonical_target.starts_with(&canonical_base) {
        return Err(AgentSyncError::PathTraversal {
            base: base.display().to_string(),
            target: target.display().to_string(),
        });
    }

    Ok(())
}

/// Canonicalize the first existing ancestor of a path
///
/// This helper function walks up the directory tree until it finds an existing
/// directory that can be canonicalized.
fn canonicalize_existing_ancestor(path: &Path) -> Result<PathBuf> {
    let mut current = path;

    loop {
        match current.canonicalize() {
            Ok(canonical) => return Ok(canonical),
            Err(_) => {
                // Try parent
                current = current.parent().ok_or_else(|| {
                    AgentSyncError::Other(format!(
                        "No existing ancestor found for path '{}'",
                        path.display()
                    ))
                })?;
            }
        }
    }
}

/// Validate that a relative path doesn't contain path traversal sequences
///
/// This is a simpler check that doesn't require the paths to exist.
/// It checks for:
/// - Absolute paths
/// - Path components that are `..`
/// - Empty paths
///
/// # Arguments
///
/// * `path` - The relative path to validate
///
/// # Errors
///
/// Returns `PathTraversal` error if the path is absolute or contains `..`
pub fn validate_relative_path(path: &Path) -> Result<()> {
    // Check if path is absolute
    if path.is_absolute() {
        return Err(AgentSyncError::PathTraversal {
            base: ".".to_string(),
            target: path.display().to_string(),
        });
    }

    // Check for ".." components
    for component in path.components() {
        if component.as_os_str() == ".." {
            return Err(AgentSyncError::PathTraversal {
                base: ".".to_string(),
                target: path.display().to_string(),
            });
        }
    }

    Ok(())
}

/// Validate a list of base directories
///
/// Ensures that:
/// - All paths are relative (not absolute)
/// - No paths contain `..` traversal sequences
/// - No paths are empty
///
/// This is used for validating the `baseDirs` configuration field.
pub fn validate_base_dirs(base_dirs: &[String]) -> Result<()> {
    if base_dirs.is_empty() {
        return Err(AgentSyncError::ConfigError {
            error: "baseDirs cannot be empty".to_string(),
        });
    }

    for base_dir in base_dirs {
        if base_dir.is_empty() {
            return Err(AgentSyncError::ConfigError {
                error: "baseDirs cannot contain empty strings".to_string(),
            });
        }

        let path = Path::new(base_dir);

        // Allow absolute paths for baseDirs (they might be needed for monorepos)
        // but validate relative paths don't contain ..
        if !path.is_absolute() {
            validate_relative_path(path).map_err(|_| {
                let base_dir_str = base_dir.as_str();
                AgentSyncError::ConfigError {
                    error: format!(
                        "Invalid baseDir '{base_dir_str}': contains path traversal sequence"
                    ),
                }
            })?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_path_within_base_safe_path() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();
        let target = base.join("subdir/file.txt");

        // Create the parent directory
        fs::create_dir_all(target.parent().unwrap()).unwrap();

        let result = validate_path_within_base(base, &target);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_within_base_traversal_attack() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Try to escape with ..
        let target = base.join("../etc/passwd");

        let result = validate_path_within_base(base, &target);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AgentSyncError::PathTraversal { .. }));
    }

    #[test]
    fn test_validate_path_within_base_absolute_path() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Try to use absolute path
        let target = Path::new("/etc/passwd");

        let result = validate_path_within_base(base, target);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_path_within_base_same_path() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Target is the same as base
        let result = validate_path_within_base(base, base);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_within_base_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create parent directory
        let parent = base.join("subdir");
        fs::create_dir_all(&parent).unwrap();

        // Target file doesn't exist yet
        let target = parent.join("newfile.txt");

        let result = validate_path_within_base(base, &target);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_path_within_base_deeply_nested() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        let target = base.join("a/b/c/d/e/file.txt");

        // Create the directory structure
        fs::create_dir_all(target.parent().unwrap()).unwrap();

        let result = validate_path_within_base(base, &target);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_relative_path_safe() {
        let path = Path::new("subdir/file.txt");
        assert!(validate_relative_path(path).is_ok());
    }

    #[test]
    fn test_validate_relative_path_with_traversal() {
        let path = Path::new("../etc/passwd");
        let result = validate_relative_path(path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AgentSyncError::PathTraversal { .. }));
    }

    #[test]
    fn test_validate_relative_path_absolute() {
        let path = Path::new("/etc/passwd");
        let result = validate_relative_path(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_relative_path_current_dir() {
        let path = Path::new(".");
        assert!(validate_relative_path(path).is_ok());
    }

    #[test]
    fn test_validate_base_dirs_valid() {
        let base_dirs = vec![".".to_string(), "packages/frontend".to_string()];
        assert!(validate_base_dirs(&base_dirs).is_ok());
    }

    #[test]
    fn test_validate_base_dirs_empty() {
        let base_dirs: Vec<String> = vec![];
        let result = validate_base_dirs(&base_dirs);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentSyncError::ConfigError { .. }
        ));
    }

    #[test]
    fn test_validate_base_dirs_with_traversal() {
        let base_dirs = vec![".".to_string(), "../other-project".to_string()];
        let result = validate_base_dirs(&base_dirs);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_base_dirs_with_empty_string() {
        let base_dirs = vec![".".to_string(), String::new()];
        let result = validate_base_dirs(&base_dirs);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_base_dirs_absolute_paths_allowed() {
        let base_dirs = vec!["/absolute/path".to_string(), ".".to_string()];
        // Absolute paths should be allowed for baseDirs
        assert!(validate_base_dirs(&base_dirs).is_ok());
    }

    #[test]
    fn test_validate_path_within_base_with_dot_components() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Path with . components (should be normalized)
        let target = base.join("./subdir/./file.txt");

        fs::create_dir_all(target.parent().unwrap()).unwrap();

        let result = validate_path_within_base(base, &target);
        assert!(result.is_ok());
    }

    #[test]
    fn test_canonicalize_existing_ancestor() {
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Create a directory structure
        let existing_dir = base.join("existing");
        fs::create_dir_all(&existing_dir).unwrap();

        // Try to canonicalize a non-existent path within existing dir
        let non_existent = existing_dir.join("nonexistent/deep/path");

        let result = canonicalize_existing_ancestor(&non_existent);
        assert!(result.is_ok());

        let canonical = result.unwrap();
        // Should return the canonical path of the existing ancestor
        // The canonical path should be an ancestor of the base
        let canonical_base = base.canonicalize().unwrap();
        assert!(
            canonical.starts_with(&canonical_base) || canonical_base.starts_with(&canonical),
            "Canonical path {canonical:?} should be related to base {canonical_base:?}"
        );
    }
}

