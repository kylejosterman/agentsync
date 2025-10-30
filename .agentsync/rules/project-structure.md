---
targets: *
globs: **/*
cursor:
  alwaysApply: true
  globs: 
windsurf:
  trigger: always_on
  globs: 
copilot:
  applyTo: **
---
# AgentSync Project Structure and Core Concepts

AgentSync is a CLI tool for synchronizing AI agent rules across multiple LLM platforms (Cursor, GitHub Copilot, Windsurf).

## Core Concept

- **Single Source of Truth**: Rules live in `.agentsync/rules/*.md`
- **Bidirectional Sync**: Can import from tools or export to tools
- **Tool-Specific Metadata**: Each tool has its own frontmatter configuration

## Project Entry Points

- [src/main.rs](mdc:src/main.rs) - CLI binary entry point
- [src/lib.rs](mdc:src/lib.rs) - Library entry with `run()` function

## Code Organization

- [cli.rs](mdc:src/cli.rs) - CLI argument definitions using clap
- [commands.rs](mdc:src/commands.rs) - Command implementations (init, add)
- [sync.rs](mdc:src/sync.rs) - Core sync logic for bidirectional synchronization
- [parser.rs](mdc:src/parser.rs) - YAML frontmatter and markdown parsing
- [models.rs](mdc:src/models.rs) - Data models for rules and metadata
- [converter/](mdc:src/converter) - Convert between AgentSync format and tool-specific formats
- [processor/](mdc:src/processor) - Process tool-specific rule files
- [config.rs](mdc:src/config.rs) - Configuration file handling
- [fs.rs](mdc:src/fs.rs) - File system utilities
- [security.rs](mdc:src/security.rs) - Path validation and security checks
- [error.rs](mdc:src/error.rs) - Error types

## Configuration

[agentsync.json](mdc:agentsync.json) in project root:

```json
{
  "tools": ["cursor", "copilot", "windsurf"],
  "baseDirs": ["."]
}
```

## Rule Format

Rules in `.agentsync/rules/*.md` have YAML frontmatter with:

- `targets`: Which tools receive this rule (`["*"]` or specific tools)
- `description`: For intelligent agent-based triggering
- `globs`: File patterns for rule application
- Tool-specific sections (`cursor:`, `windsurf:`, `copilot:`)

## Tool-Specific Output Formats

1. **Cursor** → `.cursor/rules/*.mdc`
   - Metadata: `alwaysApply`, `description`, `globs`
   - Extension: `.mdc`

2. **Windsurf** → `.windsurf/rules/*.md`
   - Metadata: `trigger` (manual, always_on, model_decision, glob), `globs`, `description`
   - Extension: `.md`

3. **GitHub Copilot** → `.github/instructions/*.md`
   - Special filename: `*.instructions.md`
   - Metadata: `applyTo` (glob pattern)
   - Extension: `.instructions.md`
