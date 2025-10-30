---
trigger: glob
description: Synchronization logic and bidirectional sync operations
globs: src/sync.rs
---
# Synchronization Logic

## Sync Module ([sync.rs](mdc:src/sync.rs))

Handles bidirectional synchronization between AgentSync and tool-specific formats.

## Sync Options

```rust
pub struct SyncOptions {
    pub dry_run: bool,
    pub verbose: bool,
}
```

## Main Sync Operations

### 1. Sync TO Tools (`sync_to_tools`)

From `.agentsync/rules/*.md` → tool-specific directories

**Process:**

1. Read all files from `.agentsync/rules/`
2. Parse each rule file
3. For each enabled tool:
   - Check if tool is in rule's `targets`
   - Convert to tool-specific format
   - Write to tool directory
4. Detect and handle deletions
5. Return summary of operations

**Tool Directories:**

- Cursor: `.cursor/rules/`
- Windsurf: `.windsurf/rules/`
- Copilot: `.github/instructions/`

### 2. Sync FROM Tool (`sync_from_tool`)

From tool-specific directory → `.agentsync/rules/*.md`

**Process:**

1. Read all files from tool directory
2. Process each tool-specific file
3. Convert to AgentSync format
4. Merge with existing rules (if any)
5. Write to `.agentsync/rules/`
6. Return summary of operations

## Sync Result

```rust
pub struct SyncResult {
    pub created: Vec<PathBuf>,
    pub updated: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
    pub errors: Vec<String>,
}

impl SyncResult {
    pub fn print_summary(&self, dry_run: bool);
}
```

## Conflict Resolution

When syncing FROM tools to AgentSync:

- **New Files**: Create new rule in `.agentsync/rules/`
- **Existing Files**: Merge tool-specific metadata, preserve other tool configs
- **Content Changes**: Warn user if content differs, prefer AgentSync version

## Dry Run Mode

When `dry_run: true`:

- Perform all validation and conversion
- Don't write any files
- Collect all operations that would be performed
- Display with "Would" prefix in summary

## File Detection

Track changes by:

- Comparing file modification times
- Computing content hashes
- Tracking which files exist in both locations

## Error Handling

Continue processing on errors:

- Collect all errors in `SyncResult.errors`
- Log individual file errors with context
- Don't fail entire sync for one file failure
- Display all errors in summary

## Performance Considerations

- Process files in parallel where possible (use `rayon` if needed)
- Cache file hashes to avoid redundant I/O
- Only write files that have changed
