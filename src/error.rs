//! Error types for AgentSync operations.

use itertools::Itertools;
use owo_colors::OwoColorize;
use thiserror::Error;

/// The main error type for AgentSync operations
#[derive(Error, Debug)]
pub enum AgentSyncError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error(
        "Configuration file not found: {path}\n\n{hint}{colon} Run {cmd} to create a new configuration file\n{hint}{colon} You can also specify a custom config with {flag}",
        path = path.cyan(),
        hint = "hint".cyan().bold(),
        colon = ":".bold(),
        cmd = "`agentsync init`".green(),
        flag = "`--config <path>`".green()
    )]
    ConfigNotFound { path: String },

    /// Custom Display for fuzzy matching suggestions
    #[error("{}", format_invalid_tool(tool))]
    InvalidTool { tool: String },

    /// Custom Display for formatted frontmatter parse error
    #[error("{}", format_frontmatter_parse_error(file, line.as_ref(), message))]
    FrontmatterParse {
        file: String,
        line: Option<usize>,
        message: String,
    },

    #[error(
        "Project not initialized\n\n{hint}{colon} Run {cmd} to initialize a new project\n{hint}{colon} This will create {config} and {dir}",
        hint = "hint".cyan().bold(),
        colon = ":".bold(),
        cmd = "`agentsync init`".green(),
        config = "`agentsync.json`".cyan(),
        dir = "`.agentsync/`".cyan()
    )]
    NotInitialized,

    /// Custom Display for platform-specific hints
    #[error("{}", format_permission_denied(path))]
    PermissionDenied { path: String },

    #[error(
        "Invalid rule name: {name}\n\n{hint}{colon} Rule names must use kebab-case (lowercase with hyphens)\n{hint}{colon} Example: {ex1}",
        name = name.red().bold(),
        hint = "hint".cyan().bold(),
        colon = ":".bold(),
        ex1 = "`my-rule`".green()
    )]
    InvalidRuleName { name: String },

    #[error("Glob pattern error: {0}")]
    GlobPattern(#[from] glob::PatternError),

    #[error(
        "Configuration error: {error}\n\n{hint}{colon} Check {config} for valid JSON syntax and field names\n{hint}{colon} Run {cmd} to validate your configuration",
        error = error.red(),
        hint = "hint".cyan().bold(),
        colon = ":".bold(),
        config = "`agentsync.json`".cyan(),
        cmd = "`agentsync validate`".green()
    )]
    ConfigError { error: String },

    /// Custom Display for nested error formatting
    #[error("{}", format_conversion_failed(rule, from_tool, to_tool, source))]
    ConversionFailed {
        rule: String,
        from_tool: String,
        to_tool: String,
        #[source]
        source: Box<AgentSyncError>,
    },

    #[error(
        "Path traversal detected: '{target}' escapes base directory '{base}'\n\nPaths must stay within the base directory."
    )]
    PathTraversal { base: String, target: String },

    #[error("{0}")]
    Other(String),
}

// Formatting functions for complex error messages

fn format_invalid_tool(tool: &str) -> String {
    let mut msg = format!("Invalid tool name: {}", tool.red().bold());

    // Find closest valid tool using Levenshtein distance
    let valid_tools = ["cursor", "copilot", "windsurf"];
    let suggestion = valid_tools
        .iter()
        .min_by_key(|valid| strsim::levenshtein(tool, valid));

    #[allow(clippy::format_push_string)]
    {
        if let Some(suggested) = suggestion
            && tool.len() > 2
        {
            msg.push_str(&format!(
                "\n\n{}{} Did you mean {}?",
                "hint".cyan().bold(),
                ":".bold(),
                suggested.green()
            ));
        }

        msg.push_str(&format!(
            "\n{}{} Valid tools are: {}",
            "hint".cyan().bold(),
            ":".bold(),
            valid_tools.iter().map(|t| t.cyan()).format(", ")
        ));
    }

    msg
}

fn format_permission_denied(path: &str) -> String {
    let mut msg = format!("Permission denied: {}", path.cyan());

    #[allow(clippy::format_push_string)]
    {
        msg.push_str(&format!(
            "\n\n{}{} The file or directory cannot be accessed due to insufficient permissions",
            "hint".cyan().bold(),
            ":".bold()
        ));

        #[cfg(unix)]
        msg.push_str(&format!(
            "\n{}{} Try running {} or {}",
            "hint".cyan().bold(),
            ":".bold(),
            "`chmod +r <path>`".green(),
            "`sudo agentsync ...`".green()
        ));

        #[cfg(windows)]
        msg.push_str(&format!(
            "\n{}{} Check the file properties and ensure you have read permissions",
            "hint".cyan().bold(),
            ":".bold()
        ));
    }

    msg
}

fn format_frontmatter_parse_error(file: &str, line: Option<&usize>, message: &str) -> String {
    let mut msg = format!("Invalid frontmatter in {}", file.cyan());

    #[allow(clippy::format_push_string)]
    {
        if let Some(line_num) = line {
            msg.push_str(&format!(" at {}", format!("line {line_num}").yellow()));
        }

        msg.push_str(&format!(
            "\n\n{}\n  {}",
            "[parse error]".red().bold(),
            message.replace('\n', "\n  ")
        ));

        msg.push_str(&format!(
            "\n\n{}{} Frontmatter must be valid key-value pairs enclosed in {} markers",
            "hint".cyan().bold(),
            ":".bold(),
            "`---`".green()
        ));

        msg.push_str(&format!(
            "\n{}{} Example format:\n  {}\n  {}\n  {}\n  {}\n  {}",
            "hint".cyan().bold(),
            ":".bold(),
            "---".green(),
            "description: My rule description".green(),
            "alwaysApply: false".green(),
            "globs: **/*.rs".green(),
            "---".green()
        ));
    }

    msg
}

fn format_conversion_failed(
    rule: &str,
    from_tool: &str,
    to_tool: &str,
    source: &AgentSyncError,
) -> String {
    let mut msg = format!(
        "Failed to convert rule {} from {} to {}",
        rule.cyan(),
        from_tool.yellow(),
        to_tool.yellow()
    );

    #[allow(clippy::format_push_string)]
    {
        msg.push_str(&format!(
            "\n\n{}\n  {}",
            "[error]".red().bold(),
            source.to_string().replace('\n', "\n  ")
        ));

        msg.push_str(&format!(
            "\n\n{}{} The rule file may contain syntax specific to {}",
            "hint".cyan().bold(),
            ":".bold(),
            from_tool.cyan()
        ));

        msg.push_str(&format!(
            "\n{}{} Try validating the source file with {} first",
            "hint".cyan().bold(),
            ":".bold(),
            format!("`agentsync validate --tool {from_tool}`").green()
        ));
    }

    msg
}

pub type Result<T> = std::result::Result<T, AgentSyncError>;
