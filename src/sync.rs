//! Bidirectional sync engine for AgentSync ↔ tool formats.

use crate::fs::{
    Tool, discover_rules, extract_rule_name, read_rule_file, rule_path, write_rule_file,
};
use crate::models::AgentSyncRule;
use crate::parser::{parse_frontmatter, serialize_frontmatter};
use crate::processor::get_processor;
use crate::{AgentSyncError, Result};
use std::path::Path;
use tracing::{debug, info};

/// Options for sync operations
#[derive(Debug, Clone, Default)]
pub struct SyncOptions {
    pub dry_run: bool,
    pub verbose: bool,
}

/// Result of a sync operation
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    pub added: Vec<String>,
    pub updated: Vec<String>,
    pub skipped: Vec<String>,
    /// (rule name, error message)
    pub errors: Vec<(String, String)>,
}

impl SyncResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn total_processed(&self) -> usize {
        self.added.len() + self.updated.len() + self.skipped.len()
    }

    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.updated.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Print sync summary
    #[allow(clippy::print_stdout)] // This is user-facing output, not debug logging
    pub fn print_summary(&self, dry_run: bool) {
        let prefix = if dry_run { "[DRY RUN] " } else { "" };

        if self.has_changes() {
            if !self.added.is_empty() {
                println!("\n{}✓ Added {} rule(s):", prefix, self.added.len());
                for rule in &self.added {
                    println!("  + {rule}");
                }
            }

            if !self.updated.is_empty() {
                println!("\n{}✓ Updated {} rule(s):", prefix, self.updated.len());
                for rule in &self.updated {
                    println!("  ~ {rule}");
                }
            }
        }

        if !self.skipped.is_empty() {
            println!(
                "\n{}→ Skipped {} rule(s) (already up-to-date)",
                prefix,
                self.skipped.len()
            );
        }

        if self.has_errors() {
            println!("\n{}✗ Errors in {} rule(s):", prefix, self.errors.len());
            for (rule, error) in &self.errors {
                println!("  ! {rule}: {error}");
            }
        }

        if !self.has_changes() && !self.has_errors() {
            println!("{prefix}✓ All rules are up-to-date");
        }

        if dry_run && self.has_changes() {
            println!("\nNo files were modified (dry-run mode)");
        }
    }
}

/// Sync rules from AgentSync format to all enabled tools
pub fn sync_to_tools(
    project_root: &Path,
    enabled_tools: &[String],
    options: &SyncOptions,
) -> Result<SyncResult> {
    info!("Starting sync from AgentSync to tools");
    let mut result = SyncResult::new();

    let agentsync_rules = discover_rules(project_root, Tool::AgentSync)?;
    debug!("Found {} AgentSync rule(s)", agentsync_rules.len());

    if agentsync_rules.is_empty() {
        info!("No rules found in .agentsync/rules/");
        return Ok(result);
    }

    // Process each AgentSync rule
    for rule_path in agentsync_rules {
        let Some(rule_name) = extract_rule_name(&rule_path) else {
            result.errors.push((
                rule_path.display().to_string(),
                "Invalid rule name".to_string(),
            ));
            continue;
        };

        debug!("Processing rule: {rule_name}");

        // Read and parse the AgentSync rule
        let content = match read_rule_file(&rule_path) {
            Ok(c) => c,
            Err(e) => {
                result.errors.push((rule_name.clone(), e.to_string()));
                continue;
            }
        };

        let agentsync_rule = match parse_frontmatter::<AgentSyncRule>(
            &content,
            Some(&rule_path.display().to_string()),
        ) {
            Ok(r) => r,
            Err(e) => {
                result.errors.push((rule_name.clone(), e.to_string()));
                continue;
            }
        };

        // Check if rule targets all tools or specific tools
        let targets_all = agentsync_rule
            .frontmatter
            .targets
            .contains(&"*".to_string());

        // Sync to each enabled tool
        for tool_name in enabled_tools {
            // Skip if rule doesn't target this tool
            if !targets_all && !agentsync_rule.frontmatter.targets.contains(tool_name) {
                continue;
            }

            let tool: Tool = match tool_name.parse() {
                Ok(t) => t,
                Err(e) => {
                    result
                        .errors
                        .push((rule_name.clone(), format!("Invalid tool: {e}")));
                    continue;
                }
            };

            if let Err(e) = sync_rule_to_tool(
                project_root,
                &rule_name,
                &agentsync_rule,
                tool,
                options,
                &mut result,
            ) {
                result
                    .errors
                    .push((format!("{rule_name} ({tool_name})"), e.to_string()));
            }
        }
    }

    Ok(result)
}

/// Sync a single AgentSync rule to a tool
fn sync_rule_to_tool(
    project_root: &Path,
    rule_name: &str,
    agentsync_rule: &crate::models::Rule<AgentSyncRule>,
    tool: Tool,
    options: &SyncOptions,
    result: &mut SyncResult,
) -> Result<()> {
    let processor = get_processor(tool);
    let tool_path = processor.rule_path(project_root, rule_name)?;
    let tool_name = tool.name();
    let full_name = format!("{rule_name} ({tool_name})");
    let tool_content = processor.convert_from_agentsync(agentsync_rule)?;

    // Check if file exists and compare content
    let is_new = !tool_path.exists();
    let needs_update = if is_new {
        true
    } else {
        let existing_content = read_rule_file(&tool_path)?;
        existing_content != tool_content
    };

    if !needs_update {
        result.skipped.push(full_name);
        return Ok(());
    }

    if !options.dry_run {
        processor.write_rule(&tool_path, &tool_content)?;
    }

    if is_new {
        if options.verbose {
            info!("Added {full_name}");
        }
        result.added.push(full_name);
    } else {
        if options.verbose {
            info!("Updated {full_name}");
        }
        result.updated.push(full_name);
    }

    Ok(())
}

/// Sync rules from a tool to AgentSync
pub fn sync_from_tool(
    project_root: &Path,
    tool: Tool,
    options: &SyncOptions,
) -> Result<SyncResult> {
    info!("Starting sync from {tool:?} to AgentSync");
    let mut result = SyncResult::new();

    if tool == Tool::AgentSync {
        return Err(AgentSyncError::Other(
            "Cannot sync from AgentSync to AgentSync".to_string(),
        ));
    }

    let processor = get_processor(tool);
    let tool_rules = processor.discover_rules(project_root)?;
    debug!("Found {} rule(s) from {:?}", tool_rules.len(), tool);

    if tool_rules.is_empty() {
        info!("No rules found for {tool:?}");
        return Ok(result);
    }

    // Process each tool rule
    for tool_rule_path in tool_rules {
        let Some(rule_name) = extract_rule_name(&tool_rule_path) else {
            result.errors.push((
                tool_rule_path.display().to_string(),
                "Invalid rule name".to_string(),
            ));
            continue;
        };

        debug!("Processing rule: {rule_name}");

        // Read and parse the tool rule
        let content = match read_rule_file(&tool_rule_path) {
            Ok(c) => c,
            Err(e) => {
                result.errors.push((rule_name.clone(), e.to_string()));
                continue;
            }
        };

        // Convert tool rule to AgentSync format
        let agentsync_rule =
            match processor.convert_to_agentsync(&content, &tool_rule_path.display().to_string()) {
                Ok(rule) => rule,
                Err(e) => {
                    result.errors.push((rule_name.clone(), e.to_string()));
                    continue;
                }
            };

        // Write to AgentSync directory
        let agentsync_path = rule_path(project_root, Tool::AgentSync, &rule_name)?;
        let agentsync_content = serialize_frontmatter(&agentsync_rule)?;

        // Check if file exists and compare content
        let is_new = !agentsync_path.exists();
        let needs_update = if is_new {
            true
        } else {
            let existing_content = read_rule_file(&agentsync_path)?;
            existing_content != agentsync_content
        };

        if !needs_update {
            result.skipped.push(rule_name.clone());
            continue;
        }

        if !options.dry_run {
            write_rule_file(&agentsync_path, &agentsync_content)?;
        }

        if is_new {
            if options.verbose {
                info!("Added {rule_name}");
            }
            result.added.push(rule_name.clone());
        } else {
            if options.verbose {
                info!("Updated {rule_name}");
            }
            result.updated.push(rule_name.clone());
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_options_default() {
        let options = SyncOptions::default();
        assert!(!options.dry_run);
        assert!(!options.verbose);
    }

    #[test]
    fn test_sync_result_new() {
        let result = SyncResult::new();
        assert_eq!(result.total_processed(), 0);
        assert!(!result.has_changes());
        assert!(!result.has_errors());
    }

    #[test]
    fn test_sync_result_with_data() {
        let mut result = SyncResult::new();
        result.added.push("rule1".to_string());
        result.updated.push("rule2".to_string());
        result.skipped.push("rule3".to_string());

        assert_eq!(result.total_processed(), 3);
        assert!(result.has_changes());
        assert!(!result.has_errors());
    }

    #[test]
    fn test_sync_result_with_errors() {
        let mut result = SyncResult::new();
        result
            .errors
            .push(("rule1".to_string(), "error message".to_string()));

        assert_eq!(result.total_processed(), 0);
        assert!(!result.has_changes());
        assert!(result.has_errors());
    }
}
