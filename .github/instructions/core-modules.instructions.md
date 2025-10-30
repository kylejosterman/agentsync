---
description: Core data structures and utilities (config, models, errors)
applyTo: src/config.rs,src/models.rs,src/error.rs
---
# Core Modules

## Configuration ([config.rs](mdc:src/config.rs))

### Config Structure

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub tools: Vec<String>,
    pub base_dirs: Vec<String>,
}
```

### Validation

```rust
impl Config {
    pub fn validate(&self) -> Result<()> {
        // Check tools list not empty
        // Check all tools are valid (cursor, copilot, windsurf)
        // Check base dirs not empty
    }
}
```

Config errors should be descriptive: "agentsync.json not found. Run 'agentsync init' first."

## Data Models ([models.rs](mdc:src/models.rs))

### AgentSync Rule Structure

```rust
pub struct AgentSyncRule {
    pub targets: Vec<String>,      // ["*"] or ["cursor", "copilot"]
    pub description: Option<String>,
    pub globs: Option<String>,
    pub cursor: Option<CursorConfig>,
    pub windsurf: Option<WindsurfConfig>,
    pub copilot: Option<CopilotConfig>,
    pub content: String,
}
```

### Tool Configurations

```rust
pub struct CursorConfig {
    pub always_apply: Option<bool>,
    pub description: Option<String>,
    pub globs: Option<String>,
}

pub struct WindsurfConfig {
    pub trigger: Option<String>,    // manual | always_on | model_decision | glob
    pub globs: Option<String>,
    pub description: Option<String>,
}

pub struct CopilotConfig {
    pub apply_to: Option<String>,   // Glob pattern
}
```

Use `#[serde(skip_serializing_if = "Option::is_none")]` for clean YAML output.

## Error Handling ([error.rs](mdc:src/error.rs))

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum AgentSyncError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration file not found: {0}")]
    ConfigNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Invalid tool: {0}")]
    InvalidTool(String),

    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("Invalid rule format: {0}")]
    InvalidRuleFormat(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Security error: {0}")]
    SecurityError(String),

    #[error("Path traversal detected: {0}")]
    PathTraversal(String),
}

pub type Result<T> = std::result::Result<T, AgentSyncError>;
```

### Error Patterns

**Propagate with context:**

```rust
let content = fs::read_to_string(&path)
    .map_err(|_| AgentSyncError::RuleNotFound(name.to_string()))?;
```

**Handle and continue:**

```rust
let mut errors = Vec::new();
for file in files {
    if let Err(e) = process_file(&file) {
        errors.push(format!("{}: {}", file, e));
        continue;
    }
}
```

**User-facing errors should:**

- Be specific: "Rule 'python' not found" not "Rule not found"
- Suggest fixes: "Run 'agentsync init' to create configuration"
- Include context: File paths, line numbers, field names
