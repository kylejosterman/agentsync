//! Processor pattern for tool-specific rule handling
//!
//! This module implements a trait-based processor pattern inspired by rulesync's
//! `FeatureProcessor` architecture. Each tool (Cursor, Copilot, Windsurf) has its
//! own processor that encapsulates all tool-specific logic.
//!
//! # Example: Adding a New Tool
//!
//! To add support for a new tool:
//!
//! 1. Add the tool variant to `Tool` enum in `fs.rs`
//! 2. Create a new processor struct implementing `Processor` trait
//! 3. Implement conversion logic (from/to AgentSync format)
//! 4. Add tests for the new processor
//! 5. Register the processor in the sync engine
//!
//! ```rust,ignore
//! pub struct NewToolProcessor;
//!
//! impl Processor for NewToolProcessor {
//!     fn tool(&self) -> Tool {
//!         Tool::NewTool
//!     }
//!
//!     fn convert_from_agentsync(&self, rule: &Rule<AgentSyncRule>) -> Result<String> {
//!         // Convert AgentSync format to NewTool format
//!         todo!()
//!     }
//!
//!     fn convert_to_agentsync(&self, content: &str, path: &str) -> Result<Rule<AgentSyncRule>> {
//!         // Convert NewTool format to AgentSync format
//!         todo!()
//!     }
//! }
//! ```

use crate::models::{AgentSyncRule, Rule};
use crate::{fs::Tool, Result};
use std::path::{Path, PathBuf};

mod cursor;
mod copilot;
mod windsurf;

pub use cursor::CursorProcessor;
pub use copilot::CopilotProcessor;
pub use windsurf::WindsurfProcessor;

/// Trait for tool-specific rule processing
///
/// Each tool (Cursor, Copilot, Windsurf) implements this trait to handle:
/// - Bidirectional format conversion (AgentSync ↔ Tool)
/// - Rule discovery in tool directories
/// - Rule file writing with tool-specific extensions
///
/// This trait provides a unified interface for the sync engine, making it
/// easy to add new tools without modifying the sync logic.
pub trait Processor {
    /// Get the tool this processor handles
    fn tool(&self) -> Tool;

    /// Convert from AgentSync format to tool-specific format
    ///
    /// Takes an AgentSync rule and converts it to the tool's native format,
    /// returning the serialized content ready to be written to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The rule contains invalid data for this tool
    /// - Serialization fails
    fn convert_from_agentsync(&self, rule: &Rule<AgentSyncRule>) -> Result<String>;

    /// Convert from tool-specific format to AgentSync format
    ///
    /// Takes the content of a tool-specific rule file and converts it to
    /// the canonical AgentSync format.
    ///
    /// # Arguments
    ///
    /// * `content` - The raw content of the tool's rule file
    /// * `path` - The file path (used for error messages)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The content cannot be parsed
    /// - The frontmatter is invalid
    /// - Required fields are missing
    fn convert_to_agentsync(&self, content: &str, path: &str) -> Result<Rule<AgentSyncRule>>;

    /// Discover all rules for this tool in the project
    ///
    /// Searches the tool's directory for rule files with the correct extension.
    /// Returns an empty vector if the directory doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The glob pattern is invalid
    /// - Directory access fails due to permissions
    fn discover_rules(&self, project_root: &Path) -> Result<Vec<PathBuf>> {
        let tool = self.tool();
        crate::fs::discover_rules(project_root, tool)
    }

    /// Write a rule file for this tool
    ///
    /// Writes content to a rule file, creating parent directories if needed.
    /// Uses atomic writes to prevent corruption.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Parent directory cannot be created
    /// - File cannot be written due to permissions
    /// - Atomic write fails
    fn write_rule(&self, path: &Path, content: &str) -> Result<()> {
        crate::fs::write_rule_file(path, content)
    }

    /// Get the full path for a rule file
    ///
    /// Constructs the path: `<project_root>/<tool_dir>/<rule_name>.<ext>`
    fn rule_path(&self, project_root: &Path, rule_name: &str) -> Result<PathBuf> {
        crate::fs::rule_path(project_root, self.tool(), rule_name)
    }
}

/// Get a processor for a specific tool
///
/// This function acts as a factory for processors, returning the appropriate
/// processor implementation for the given tool.
///
/// # Example
///
/// ```rust,ignore
/// let processor = get_processor(Tool::Cursor);
/// let agentsync_rule = processor.convert_to_agentsync(&content, &path)?;
/// ```
#[must_use]
pub fn get_processor(tool: Tool) -> Box<dyn Processor> {
    match tool {
        Tool::Cursor => Box::new(CursorProcessor),
        Tool::Copilot => Box::new(CopilotProcessor),
        Tool::Windsurf => Box::new(WindsurfProcessor),
        Tool::AgentSync => {
            // AgentSync doesn't need a processor since it's the canonical format
            unreachable!("AgentSync tool does not have a processor")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_processor_cursor() {
        let processor = get_processor(Tool::Cursor);
        assert_eq!(processor.tool(), Tool::Cursor);
    }

    #[test]
    fn test_get_processor_copilot() {
        let processor = get_processor(Tool::Copilot);
        assert_eq!(processor.tool(), Tool::Copilot);
    }

    #[test]
    fn test_get_processor_windsurf() {
        let processor = get_processor(Tool::Windsurf);
        assert_eq!(processor.tool(), Tool::Windsurf);
    }

    #[test]
    #[should_panic(expected = "AgentSync tool does not have a processor")]
    fn test_get_processor_agentsync_panics() {
        let _processor = get_processor(Tool::AgentSync);
    }
}

