//! CLI tool for synchronizing AI agent rules files.
//!
//! Maintains a single source of truth in `.agentsync/rules/` and syncs
//! bidirectionally with Cursor, GitHub Copilot, and Windsurf.

// Allow println/print in this module as it's a CLI tool
#![allow(clippy::print_stdout)]

pub mod cli;
pub mod commands;
pub mod config;
pub mod converter;
pub mod error;
pub mod fs;
pub mod models;
pub mod parser;
pub mod processor;
pub mod security;
pub mod sync;

pub use cli::{Cli, Commands};
pub use error::{AgentSyncError, Result};

use tracing::{debug, info};

/// Run the application with parsed CLI arguments
pub fn run(args: Cli) -> Result<()> {
    // Initialize logging
    init_logging(args.verbose);

    debug!("Starting agentsync with args: {args:?}");

    match args.command {
        Commands::Init => {
            info!("Running init command");
            commands::run_init(args.verbose)
        }
        Commands::Sync { from, dry_run } => {
            // Create sync options
            let options = sync::SyncOptions {
                dry_run,
                verbose: args.verbose,
            };

            if let Some(tool_name) = from {
                // Sync to AgentSync
                info!("Running sync --from {tool_name}");

                let project_root = fs::find_project_root()?;
                let tool: fs::Tool = tool_name.parse()?;

                println!("Syncing from {tool_name} to .agentsync/rules/...");
                let result = sync::sync_from_tool(&project_root, tool, &options)?;
                result.print_summary(dry_run);
            } else {
                // Sync from AgentSync
                info!("Running sync to tools");

                let project_root = fs::find_project_root()?;
                let config = config::load_config(project_root.join("agentsync.json"))?;
                config.validate()?;

                println!("Syncing from .agentsync/rules/ to enabled tools...");
                let result = sync::sync_to_tools(&project_root, &config.tools, &options)?;
                result.print_summary(dry_run);
            }
            Ok(())
        }
        Commands::Add { name } => {
            info!("Running add command for rule: {name}");
            commands::run_add(&name, args.verbose)
        }
    }
}

fn init_logging(verbose: bool) {
    use tracing_subscriber::{EnvFilter, fmt};

    let log_level = if verbose { "debug" } else { "warn" };

    // Use try_init to avoid panicking if logger is already initialized (e.g., in tests)
    let _ = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level)),
        )
        .without_time()
        .with_target(false)
        .try_init();
}
