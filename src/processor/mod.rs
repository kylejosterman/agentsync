//! Tool-specific processors for rule handling (Cursor, Copilot, Windsurf).

use crate::models::{AgentSyncRule, Rule};
use crate::{Result, fs::Tool};
use std::path::{Path, PathBuf};

mod copilot;
mod cursor;
mod windsurf;

pub use copilot::CopilotProcessor;
pub use cursor::CursorProcessor;
pub use windsurf::WindsurfProcessor;

/// Tool-specific processor trait for bidirectional conversion and file operations
pub trait Processor {
    fn tool(&self) -> Tool;

    /// Convert AgentSync to tool format
    fn convert_from_agentsync(&self, rule: &Rule<AgentSyncRule>) -> Result<String>;

    /// Convert tool format to AgentSync
    fn convert_to_agentsync(&self, content: &str, path: &str) -> Result<Rule<AgentSyncRule>>;

    fn discover_rules(&self, project_root: &Path) -> Result<Vec<PathBuf>> {
        crate::fs::discover_rules(project_root, self.tool())
    }

    fn write_rule(&self, path: &Path, content: &str) -> Result<()> {
        crate::fs::write_rule_file(path, content)
    }

    fn rule_path(&self, project_root: &Path, rule_name: &str) -> Result<PathBuf> {
        crate::fs::rule_path(project_root, self.tool(), rule_name)
    }
}

/// Get processor for tool
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
