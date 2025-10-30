---
targets: *
description: General Rust best practices, idioms, and patterns
globs: **/*.rs
cursor:
  alwaysApply: false
  globs: **/*.rs
windsurf:
  trigger: glob
  globs: **/*.rs
copilot:
  applyTo: **/*.rs
---
# Rust Best Practices and Idiomatic Patterns

## Key Principles

- Write only what is requested, do not over engineer, do not write overly defensive code
- Write clear, concise, and idiomatic Rust code with accurate examples
- Always focus on adhering to Rust best practices and use documentation when available
- Prioritize modularity, clean code organization, and efficient resource management
- Use expressive variable names that convey intent (e.g., `is_ready`, `has_data`)
- Adhere to Rust's naming conventions: snake_case for variables and functions, PascalCase for types and structs
- Avoid code duplication; use functions and modules to encapsulate reusable logic
- Write code with safety, concurrency, and performance in mind, embracing Rust's ownership and type system
- Prefer standard library over external dependencies when sufficient
- Run `cargo clippy` regularly and address warnings
- Format code with `rustfmt` before committing

## Error Handling

- Use `Result<T, E>` for operations that can fail
- Use `thiserror` for library error types with custom context
- Use `anyhow` for application-level error handling
- Prefer `?` operator over `unwrap()` or `expect()` in library code
- Use `Option<T>` for absence of values, not for error cases
- In production code, avoid `unwrap()` and `expect()`
- In tests, use `.expect("descriptive message")` over `.unwrap()`

## Testing

- Unit tests: `#[cfg(test)]` modules in same file
- Integration tests: `tests/` directory for public API testing
- Use `assert!`, `assert_eq!`, `assert_ne!` appropriately
- In tests, use `.expect("descriptive message")` instead of `.unwrap()`
- Test error cases, not just happy paths
- Use descriptive test names that explain what's being tested

## Ownership and Borrowing

- Prefer borrowing (`&T`) over moving when possible
- Use `&mut T` when you need to modify, `&T` for read-only access
- Return owned types from functions when caller needs ownership
- Use `.clone()` judiciously - understand the cost
- Know the difference: `&str` vs `String`, `&[T]` vs `Vec<T>`, `&Path` vs `PathBuf`

## Type System and Enums

- Leverage enums for variants and state machines
- Use exhaustive pattern matching with `match`
- Use `if let` and `while let` for single-pattern matches
- Consider newtypes for type safety: `struct UserId(u64)`
- Implement `From`/`Into` for ergonomic type conversions
- Derive common traits: `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`

## Iterators and Collections

- Prefer iterator chains over manual loops
- Use `.iter()` (borrow), `.iter_mut()` (mutable), `.into_iter()` (consume)
- Chain methods: `map()`, `filter()`, `fold()`, `find()`, `collect()`
- Avoid collecting intermediate values unnecessarily
- Choose the right collection: `Vec`, `HashMap`, `HashSet`, `BTreeMap`

## Documentation

- Document public APIs with `///` doc comments
- Include examples in doc comments (they're automatically tested)
- Use `//!` for module-level documentation
- Add `# Examples`, `# Errors`, `# Panics` sections when relevant
- Run `cargo doc --open` to preview generated documentation

## Memory and Smart Pointers

- Understand stack vs heap allocation
- Use `Box<T>` for heap allocation of single ownership
- Use `Rc<T>` for shared ownership (single-threaded)
- Use `Arc<T>` for shared ownership (thread-safe)
- Reserve capacity when size is known: `Vec::with_capacity(n)`
- Profile before optimizing - measure, don't guess

## Attributes and Markers

- Use `#[must_use]` on types/functions where ignoring return is a bug
- Derive `Debug` on all types for better error messages
- Use `#[allow(clippy::...)]` sparingly with explanation
- Avoid `unsafe` unless necessary; always document safety invariants

## Concurrency Basics

- Use `std::thread` for spawning threads
- Use channels (`std::sync::mpsc`) for message passing between threads
- Use `Mutex<T>` for shared mutable state
- Use `RwLock<T>` when many readers, few writers
- Prefer message passing over shared state when possible
