//! Path validation to prevent traversal attacks and ensure operations stay within boundaries.

use crate::{AgentSyncError, Result};
use std::path::{Path, PathBuf};

/// Validate target path is within base directory (protects against traversal and symlink attacks)
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

/// Walk up directory tree to find first existing ancestor for canonicalization
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

/// Validate relative path doesn't contain `..` or absolute paths
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

/// Validate baseDirs list (no empty, no `..` in relative paths)
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
        assert!(matches!(
            result.unwrap_err(),
            AgentSyncError::PathTraversal { .. }
        ));
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
        assert!(matches!(
            result.unwrap_err(),
            AgentSyncError::PathTraversal { .. }
        ));
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
