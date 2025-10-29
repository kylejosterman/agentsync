//! Command-line interface definitions
//!
//! This module defines the CLI structure using clap's derive macros.

use clap::{Parser, Subcommand};

/// AgentSync CLI application
#[derive(Parser, Debug)]
#[command(
    name = "agentsync",
    version,
    about = "CLI tool for synchronizing AI agent rules files across multiple platforms",
    long_about = "AgentSync maintains a single source of truth for agent rules and syncs them across Cursor, GitHub Copilot, and Windsurf."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize AgentSync in the current project
    #[command(about = "Create .agentsync/ directory and configuration")]
    Init,

    /// Sync rules between AgentSync and tools
    #[command(
        about = "Sync rules from .agentsync/rules/ to enabled tools (default) or from a specific tool"
    )]
    Sync {
        /// Sync FROM a specific tool TO .agentsync/rules/
        #[arg(long, value_name = "TOOL")]
        from: Option<String>,

        /// Preview changes without writing files
        #[arg(long, short = 'n')]
        dry_run: bool,
    },

    /// Create a new rule template
    #[command(about = "Create a new rule file in .agentsync/rules/")]
    Add {
        /// Name of the rule (kebab-case recommended)
        #[arg(value_name = "RULE_NAME")]
        name: String,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Parse arguments from an iterator (useful for testing)
    pub fn parse_args_from<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Self::parse_from(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        // Verify the CLI structure is valid
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
