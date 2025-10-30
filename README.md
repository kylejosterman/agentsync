# AgentSync

A Rust CLI tool for synchronizing AI agent rules across LLM tools.

- 🔄 **Bidirectional sync** between Cursor, GitHub Copilot, and Windsurf
- 📝 **Single source of truth** in `.agentsync/rules/`
- ⚙️ **Tool-specific configurations** preserved automatically
- ⚡ **Fast and reliable**, written in Rust

## Table of Contents

1. [Getting Started](#getting-started)
1. [Installation](#installation)
1. [Usage](#usage)
1. [Configuration](#configuration)
1. [Rule Format](#rule-format)
1. [Contributing](#contributing)
1. [License](#license)

## Getting Started

Initialize AgentSync and create your first rule:

```bash
agentsync init
agentsync add python-dev
# Edit .agentsync/rules/python-dev.md with your rule content
agentsync sync
```

If you already have rules, import them:

```bash
agentsync init  # Prompts to import existing rules
# Or import from a specific tool
agentsync sync --from cursor
```

## Installation

Install AgentSync via Homebrew:

```bash
brew tap kylejosterman/agentsync
brew install agentsync
```

### Usage

#### Initialize a project

```bash
agentsync init
```

Creates `.agentsync/rules/` directory and `agentsync.json` configuration file. If existing rules are found, you'll be prompted to import them.

**Note** Agentsync currently supports Cursor, Github Copilot and Windsurf

#### Sync rules

```bash
agentsync sync                # Sync to all enabled tools
agentsync sync --from cursor  # Import from a specific tool
agentsync sync --dry-run      # Preview changes without writing files
```

Creates a new rule template in `.agentsync/rules/<rule-name>.md`.

#### Global options

- `-v, --verbose`: Show detailed logging
- `-n, --dry-run`: Preview changes without writing files
- `-h, --help`: Show help information
- `-V, --version`: Show version

For complete command documentation, run `agentsync --help` or `agentsync <command> --help`.

## Configuration

AgentSync uses `agentsync.json` in your project root:

```json
{
  "tools": ["cursor", "copilot", "windsurf"],
  "baseDirs": ["."]
}
```

- **`tools`**: Which tools to sync with (`cursor`, `copilot`, `windsurf`)
- **`baseDirs`**: Base directories for monorepo support

## Rule Format

Rules are stored in `.agentsync/rules/*.md` with YAML frontmatter:

```yaml
---
targets: ["*"]
description: "Python development best practices"
globs: "**/*.py"

cursor:
  alwaysApply: false

windsurf:
  trigger: glob
  globs: "**/*.py"

copilot:
  applyTo: "**/*.py"
---
Your rule content here...
```

### Common fields

- **`targets`**: Which tools receive this rule (`["*"]` for all, or `["cursor", "copilot"]` for specific tools)
- **`description`**: Used by agents to determine when to apply the rule intelligently
- **`globs`**: File patterns for rule application (e.g., `"**/*.py"`, `"src/**/*.ts"`)

### Tool-specific fields

**Cursor** (`.cursor/rules/*.mdc`)

- `alwaysApply: true` — Always in context
- `alwaysApply: false` — Apply intelligently or to specific files (with globs)
- Access via `@ruleName` for manual activation

**Windsurf** (`.windsurf/rules/*.md`)

- `trigger: manual` — Activate via @mention
- `trigger: always_on` — Always in context
- `trigger: model_decision` — Agent decides based on description
- `trigger: glob` — Apply to files matching globs

**GitHub Copilot** (`.github/instructions/*.md`)

- `applyTo: "**/*.py"` — Apply to files matching glob pattern
- Always included in context when files match

### Examples

**Always apply rule:**

```yaml
---
targets: ["*"]
description: "Core coding standards"
cursor:
  alwaysApply: true
windsurf:
  trigger: always_on
---
```

**Apply to specific files:**

```yaml
---
targets: ["*"]
description: "Python best practices"
globs: "**/*.py"
cursor:
  alwaysApply: false
windsurf:
  trigger: glob
  globs: "**/*.py"
copilot:
  applyTo: "**/*.py"
---
```

**Tool-specific rule:**

```yaml
---
targets: ["cursor"]
description: "Cursor-specific rules"
cursor:
  alwaysApply: true
---
```

## Contributing

Contributions are welcome. To get started:

```bash
# Install from source
cargo install --path .

# Run tests
cargo test

# Run linter
cargo clippy --all-targets --all-features

# Format code
cargo fmt
```

## License

AgentSync is licensed under the [MIT License](LICENSE).

---
