---
trigger: glob
description: Testing conventions and best practices for AgentSync
globs: src/**/*.rs,tests/**/*.rs
---
# Testing Conventions

## Test Organization

1. **Unit Tests**: In same file as implementation

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_function() {
           // test code
       }
   }
   ```

2. **Integration Tests**: In `tests/` directory
   - Test full workflows (init, sync, add commands)
   - Use `assert_fs` for filesystem fixtures
   - Use `predicates` for assertions

## Testing File Operations

Use `assert_fs` for temporary test directories:

```rust
use assert_fs::prelude::*;
use assert_fs::TempDir;

#[test]
fn test_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file = temp_dir.child("test.txt");

    // Perform operations
    file.write_str("content").unwrap();

    // Assert
    file.assert("content");

    // Cleanup is automatic when temp_dir drops
}
```

## Testing with Predicates

Use `predicates` for flexible file content assertions:

```rust
use predicates::prelude::*;

file.assert(predicate::str::contains("expected text"));
file.assert(predicate::path::exists());
```

## Test Data

- Create minimal test fixtures
- Use realistic rule examples
- Test both valid and invalid inputs
- Test edge cases (empty files, malformed YAML, missing fields)

## Testing Error Cases

Test error handling thoroughly:

```rust
#[test]
fn test_invalid_config() {
    let result = load_config(invalid_path);
    assert!(result.is_err());

    match result.unwrap_err() {
        AgentSyncError::ConfigNotFound => (),
        _ => panic!("Expected ConfigNotFound error"),
    }
}
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_test_name
```

## Test Coverage Areas

- ✅ Config loading and validation
- ✅ Rule parsing (valid and invalid YAML)
- ✅ File operations (read/write/create directories)
- ✅ Path security validation
- ✅ Conversion between formats
- ✅ Sync operations (dry-run and actual)
- ✅ CLI argument parsing
- ✅ Error handling and messages
