---
description: Rules for parsing and converting between rule formats
applyTo: src/parser.rs,src/converter/**/*.rs,src/processor/**/*.rs
---
# Parsing and Converting Rule Formats

## Parser Module ([parser.rs](mdc:src/parser.rs))

The parser handles YAML frontmatter and markdown content:

- **YAML Frontmatter**: Between `---` delimiters at file start
- **Markdown Content**: Everything after the second `---`
- Uses `serde` for YAML deserialization

### Key Functions

- Parsing frontmatter into tool-specific metadata structs
- Extracting markdown content
- Validating frontmatter structure

## Converter Module ([src/converter/](mdc:src/converter))

Converts from AgentSync format (`.agentsync/rules/*.md`) to tool-specific formats:

- [converter/cursor.rs](mdc:src/converter/cursor.rs) - Convert to `.cursor/rules/*.mdc`
- [converter/windsurf.rs](mdc:src/converter/windsurf.rs) - Convert to `.windsurf/rules/*.md`
- [converter/copilot.rs](mdc:src/converter/copilot.rs) - Convert to `.github/instructions/*.instructions.md`

### Conversion Rules

1. **Cursor Conversion**
   - Read `cursor.alwaysApply` from source
   - Use top-level `description` and `globs`
   - Output: `.mdc` files with simplified frontmatter

2. **Windsurf Conversion**
   - Read `windsurf.trigger` and `windsurf.globs`
   - Map trigger types: manual, always_on, model_decision, glob
   - Output: `.md` files

3. **Copilot Conversion**
   - Read `copilot.applyTo` (glob pattern)
   - Filename must end with `.instructions.md`
   - Output: minimal frontmatter with `applyTo`

## Processor Module ([src/processor/](mdc:src/processor))

Processes tool-specific files back to AgentSync format (reverse conversion):

- [processor/cursor.rs](mdc:src/processor/cursor.rs) - Read `.cursor/rules/*.mdc`
- [processor/windsurf.rs](mdc:src/processor/windsurf.rs) - Read `.windsurf/rules/*.md`
- [processor/copilot.rs](mdc:src/processor/copilot.rs) - Read `.github/instructions/*.instructions.md`

### Processing Rules

1. Extract tool-specific metadata
2. Preserve markdown content exactly
3. Create or update corresponding `.agentsync/rules/*.md` file
4. Merge metadata into appropriate tool sections

## Important Notes

- **Preserve Content**: Never modify markdown content during conversion
- **Filename Mapping**: Strip tool-specific suffixes (`.instructions.md` → `.md`)
- **Target Filtering**: Only convert rules where tool is in `targets` array
- **Metadata Preservation**: Keep tool-specific metadata in separate sections
