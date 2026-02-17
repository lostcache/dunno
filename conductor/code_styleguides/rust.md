# Rust Code Style Guide

## Formatting
- **Standard:** Adhere strictly to the official [Rust Style Guide](https://github.com/rust-dev-tools/fmt-rfcs).
- **Tooling:** Use `rustfmt` for automatic formatting. All code must pass `cargo fmt --check`.

## Linting
- **Tooling:** Use `clippy` for linting.
- **Severity:** Treat warnings as errors in CI (`cargo clippy -- -D warnings`).
- **Idioms:** Favor idiomatic Rust (e.g., `Option`, `Result`, iterators) over C-style loops and manual memory management.

## Naming Conventions
- **Variables & Functions:** `snake_case`
- **Types & Traits:** `UpperCamelCase`
- **Constants:** `SCREAMING_SNAKE_CASE`
- **Modules:** `snake_case` (avoid `mod.rs` where possible in newer Rust editions)

## Error Handling
- **Result Type:** Use `Result<T, E>` for recoverable errors.
- **Panic:** Avoid `unwrap()` and `expect()` in production code. Use `?` operator for error propagation.
- **Custom Errors:** Define custom error types using `thiserror` (for libraries) or `anyhow` (for applications) where appropriate.

## Documentation
- **Doc Comments:** Use `///` for public API documentation.
- **Examples:** Include doctests in documentation where feasible.
- **Readme:** Ensure the crate-level documentation (or `README.md`) explains the purpose and usage.

## Testing
- **Unit Tests:** Place unit tests in a `tests` module within the same file (`#[cfg(test)]`).
- **Integration Tests:** Place integration tests in the `tests/` directory.
- **Property Testing:** Consider `proptest` for complex logic.
