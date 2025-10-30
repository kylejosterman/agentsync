//! CLI command implementations (`init`, `add`).

use crate::{AgentSyncError, Result, config, fs, sync};
use itertools::Itertools;
use tracing::info;

/// Initialize AgentSync: create directories, config, and optionally import existing rules
pub fn run_init(verbose: bool) -> Result<()> {
    use fs_err as fs;
    use std::io::{self, Write};

    let current_dir = std::env::current_dir()?;

    // Check if already initialized
    let config_path = current_dir.join("agentsync.json");
    if config_path.exists() {
        return Err(AgentSyncError::Other(
            "Project already initialized (agentsync.json exists)".to_string(),
        ));
    }

    let agentsync_dir = current_dir.join(".agentsync/rules");
    if verbose {
        info!("Creating directory: {}", agentsync_dir.display());
    }
    fs::create_dir_all(&agentsync_dir)?;
    println!("✓ Created .agentsync/rules/");

    let default_config = config::create_default_config();
    config::save_config(&config_path, &default_config)?;
    println!("✓ Created agentsync.json");

    // Scan for existing rules in tool directories
    let mut found_tools = Vec::new();
    for tool_name in &["cursor", "copilot", "windsurf"] {
        if let Ok(tool) = tool_name.parse::<crate::fs::Tool>() {
            let rules = crate::fs::discover_rules(&current_dir, tool)?;
            if !rules.is_empty() {
                found_tools.push(((*tool_name).to_string(), rules.len()));
                if verbose {
                    info!("Found {} rule(s) in {}", rules.len(), tool.directory());
                }
            }
        }
    }

    // If rules found, prompt user which to import
    if !found_tools.is_empty() {
        println!("\nFound existing rules:");
        for (tool, count) in &found_tools {
            println!("  - {tool}: {count} rule(s)");
        }

        print!("\nWhich tool to import from? [");
        for (i, (tool, _)) in found_tools.iter().enumerate() {
            if i > 0 {
                print!("/");
            }
            print!("{tool}");
        }
        print!("/skip]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim().to_lowercase();

        if choice != "skip" && !choice.is_empty() {
            let valid_choice = found_tools.iter().any(|(tool, _)| tool == &choice);
            if !valid_choice {
                return Err(AgentSyncError::Other(format!(
                    "Invalid choice '{}'. Expected one of: {}",
                    choice,
                    found_tools.iter().map(|(t, _)| t.as_str()).format(", ")
                )));
            }

            // Import from selected tool
            let tool: crate::fs::Tool = choice.parse()?;
            let options = sync::SyncOptions {
                dry_run: false,
                verbose,
            };

            let result = sync::sync_from_tool(&current_dir, tool, &options)?;
            println!("✓ Imported {} rule(s) from {}", result.added.len(), choice);

            if verbose && !result.added.is_empty() {
                for rule in &result.added {
                    info!("  - {rule}");
                }
            }
        } else {
            println!("Skipped import. You can import later with 'agentsync sync --from <tool>'");
        }
    }

    println!("\n✓ Initialization complete!");
    println!("  - Edit rules in .agentsync/rules/");
    println!("  - Run 'agentsync sync' to propagate changes to tools");

    Ok(())
}

/// Create a new rule template in `.agentsync/rules/`
pub fn run_add(name: &str, verbose: bool) -> Result<()> {
    if name.is_empty() {
        return Err(AgentSyncError::Other(
            "Rule name cannot be empty".to_string(),
        ));
    }

    // Check for path traversal in rule name
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(AgentSyncError::PathTraversal {
            base: ".agentsync/rules".to_string(),
            target: name.to_string(),
        });
    }

    // Check for invalid characters
    if name.contains(|c: char| !c.is_alphanumeric() && c != '-' && c != '_') {
        return Err(AgentSyncError::Other(
            "Rule name can only contain alphanumeric characters, hyphens, and underscores"
                .to_string(),
        ));
    }

    let project_root = fs::find_project_root()?;
    let rule_path = project_root
        .join(".agentsync/rules")
        .join(format!("{name}.md"));

    crate::security::validate_path_within_base(&project_root, &rule_path)?;

    if rule_path.exists() {
        return Err(AgentSyncError::Other(format!(
            "Rule '{}' already exists at {}",
            name,
            rule_path.display()
        )));
    }

    let template = create_rule_template(name);

    if verbose {
        info!("Creating rule file: {}", rule_path.display());
    }

    fs::write_rule_file(&rule_path, &template)?;

    println!("✓ Created .agentsync/rules/{name}.md");
    println!("Edit the rule, then run 'agentsync sync' to propagate to tools.");

    Ok(())
}

/// Generate rule template with YAML frontmatter. Converts kebab-case to Title Case.
fn create_rule_template(name: &str) -> String {
    use indoc::formatdoc;

    let title = name
        .replace('-', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .format(" ")
        .to_string();

    formatdoc! {"
        ---
        targets:
          - \"*\"
        description: \"Description of this rule\"
        globs: \"**/*\"
        cursor:
          alwaysApply: false
          globs: \"\"
        windsurf:
          trigger: model_decision
          globs: \"\"
        copilot:
          applyTo: \"**\"
        ---
        # {title}

        Your rule content here...
        "
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_rule_template() {
        let template = create_rule_template("python-dev");
        assert!(template.contains("targets:"));
        assert!(template.contains("# Python Dev"));
        assert!(template.contains("cursor:"));
        assert!(template.contains("windsurf:"));
        assert!(template.contains("copilot:"));
    }

    #[test]
    fn test_create_rule_template_single_word() {
        let template = create_rule_template("rust");
        assert!(template.contains("# Rust"));
    }

    #[test]
    fn test_create_rule_template_multiple_hyphens() {
        let template = create_rule_template("my-awesome-rule");
        assert!(template.contains("# My Awesome Rule"));
    }

    #[test]
    fn test_run_add_rejects_path_traversal() {
        // Test that path traversal attempts are rejected
        let result = run_add("../../../etc/passwd", false);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentSyncError::PathTraversal { .. }
        ));
    }

    #[test]
    fn test_run_add_rejects_forward_slash() {
        let result = run_add("subdir/rule", false);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentSyncError::PathTraversal { .. }
        ));
    }

    #[test]
    fn test_run_add_rejects_backslash() {
        let result = run_add("subdir\\rule", false);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentSyncError::PathTraversal { .. }
        ));
    }

    #[test]
    fn test_run_add_rejects_dot_dot() {
        let result = run_add("..rule", false);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AgentSyncError::PathTraversal { .. }
        ));
    }
}
