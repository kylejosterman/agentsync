# AgentSync

A Rust CLI tool for synchronizing AI agent rules across multiple platforms.

## Why AgentSync?

Maintain a **single source of truth** for your AI agent rules in `.agentsync/rules/` and sync them bidirectionally with:

- **Cursor** (`.cursor/rules/*.mdc`)
- **GitHub Copilot** (`.github/instructions/*.md`)
- **Windsurf** (`.windsurf/rules/*.md`)

**Key Features:**

- Single source of truth for all your agent rules
- Bidirectional sync: import from any tool, export to all
- Tool-specific configurations preserved automatically
- Fast and reliable (written in Rust)

## Installation

```bash
# From source (requires Rust)
cargo install --path .
```

## Quick Start

```bash
# Initialize in your project
agentsync init

# Create a new rule
agentsync add python-dev

# Edit .agentsync/rules/python-dev.md, then sync
agentsync sync

# Import existing rules from a tool
agentsync sync --from cursor
```

## Commands

### `agentsync init`

Initialize AgentSync in your project. Creates `.agentsync/rules/` and `agentsync.json`.

```bash
agentsync init
```

If existing rules are found in Cursor, Copilot, or Windsurf, you'll be prompted to import them.

### `agentsync sync`

Sync rules to all enabled tools (default) or import from a specific tool.

```bash
# Sync to all enabled tools
agentsync sync

# Import from a specific tool
agentsync sync --from cursor

# Preview changes (dry-run)
agentsync sync -n
```

### `agentsync add <name>`

Create a new rule template.

```bash
agentsync add python-dev
```

### Global Flags

- `-v, --verbose`: Detailed logging
- `-n, --dry-run`: Preview changes without writing files
- `-h, --help`: Show help

## Configuration

### `agentsync.json`

```json
{
  "tools": ["cursor", "copilot", "windsurf"],
  "baseDirs": ["."]
}
```

- **`tools`**: Which tools to sync with
- **`baseDirs`**: Base directories (for monorepo support)

### Rule Format

Rules in `.agentsync/rules/*.md` use YAML frontmatter:

```yaml
---
targets: ["*"]
description: "Python development best practices"
globs: "**/*.py"

cursor:
  alwaysApply: false
  globs: "**/*.py"

windsurf:
  trigger: glob
  globs: "**/*.py"

copilot:
  applyTo: "**/*.py"
---

# Python Development

Your rule content here...
```

### Key Fields

- **`targets`**: Which tools receive this rule (`["*"]` for all)
- **`description`**: Used by agents for intelligent application
- **`globs`**: File patterns for rule application
- **`cursor.alwaysApply`**:
  - `true` = Always Apply (always in context)
  - `false` = Apply Intelligently (when relevant) or Apply to Specific Files (with globs)
- **`windsurf.trigger`**:
  - `manual` = Activate via @mention
  - `always_on` = Always Apply
  - `model_decision` = Apply Intelligently (uses description to decide)
  - `glob` = Apply to Specific Files
- **`copilot.applyTo`**: Glob patterns for file matching (always included in context)

## Examples

### Starting Fresh

```bash
cd my-project
agentsync init
agentsync add python-dev
agentsync add react-components
# Edit rules in .agentsync/rules/
agentsync sync
```

### Importing Existing Rules

```bash
cd my-project
agentsync init  # Prompts to import
# Or: agentsync sync --from cursor
agentsync sync  # Sync to all tools
```

### Tool-Specific Rule

```yaml
---
targets: ["cursor"]
description: "Cursor-specific shortcuts"
cursor:
  alwaysApply: true
---

# Cursor Shortcuts
...
```

## How It Works

AgentSync intelligently converts between tool formats:

- **Always Apply** (`alwaysApply: true`) → Windsurf `always_on` / Copilot `**`
- **Apply Intelligently** (`alwaysApply: false`, no globs) → Windsurf `model_decision`
- **Apply to Specific Files** (with globs) → Appropriate format for each tool
- **Tool-specific settings** → Preserved during sync

## Tool-Specific Behavior

**Cursor** (`.cursor/rules/*.mdc`)

- Always Apply: `alwaysApply: true`
- Apply Intelligently: `alwaysApply: false`
- Apply to Specific Files: `alwaysApply: false` with globs
- Manual: Via `@ruleName`

**GitHub Copilot** (`.github/instructions/*.md`)

- Always included with optional `applyTo` for glob patterns

**Windsurf** (`.windsurf/rules/*.md`)

- **Manual**: Activate via @mention (`manual`)
- **Always Apply**: Always in context (`always_on`)
- **Apply Intelligently**: Model decides when to apply based on description (`model_decision`)
- **Apply to Specific Files**: Applied to files matching globs (`glob`)

## Contributing

```bash
# Run tests
cargo test

# Run linter
cargo clippy --all-targets --all-features

# Format code
cargo fmt
```

## License

MIT License - see [LICENSE](LICENSE) for details

---
