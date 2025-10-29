---
description: Rust-specific coding standards and best practices
applyTo: '**/*.rs, **/*.toml'
---

# Test Rule - Apply to Specific Files

This rule is applied only to Rust files (*.rs) and TOML files (*.toml).

It should appear in:
- Cursor: Auto Attached mode with globs (alwaysApply: false, globs: '**/*.rs, **/*.toml')
- Windsurf: glob mode (globs: '**/*.rs, **/*.toml')
- Copilot: Applied to matching files (applyTo: '**/*.rs, **/*.toml')

## Rust Guidelines

- Use `cargo fmt` for formatting
- Run `cargo clippy` for linting
- Write documentation comments with `///`
- Use `Result<T, E>` for error handling
- Prefer `&str` over `String` for function parameters

