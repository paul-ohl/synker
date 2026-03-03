# Agent Guidelines - Synker

This document provides essential information for autonomous agents operating in the `synker` repository.

## 🛠 Build, Test, and Lint Commands

This is a Rust project using the 2024 edition.

- **Build:** `cargo build`
- **Check (Fast):** `cargo check`
- **Run:** `cargo run`
- **Test All:** `cargo test`
- **Run Single Test:** `cargo test <test_name>` (e.g., `cargo test domain::logic::file_manager::tests::test_create_file`)
- **Lint:** `cargo clippy`
- **Format:** `cargo fmt`
- **Documentation:** `cargo doc --open`

## 🏗 Architecture: Hexagonal (Ports & Adapters)

The project follows a Hexagonal Architecture pattern to decouple business logic from infrastructure.

- **`src/domain`**: Core business logic, domain types, and **Ports** (traits).
    - `src/domain/types`: Pure data structures (POJOs).
    - `src/domain/services`: Trait definitions (Ports) for outbound operations.
    - `src/domain/logic`: Orchestrators that implement business rules and delegate to ports.
- **`src/inbound`**: Inbound adapters (e.g., REST API with Axum).
    - **`src/inbound/server/handlers/`**: Each route has its own handler file.
- **`src/outbound`**: Outbound adapters (e.g., File System implementation of `FileManager`).

## 🖋 Code Style & Conventions

### 1. Naming Conventions
- **Files/Modules:** `snake_case.rs`
- **Structs/Enums/Traits:** `PascalCase`
- **Functions/Variables/Fields:** `snake_case`
- **Constants:** `SCREAMING_SNAKE_CASE`

### 2. Imports & Module Structure
- Group imports: standard library first, then external crates, then local modules.
- Use explicit paths for local modules: `use crate::domain::...`.
- Avoid `use super::*;` unless in a test module.
- Keep `mod.rs` files for re-exporting internal module contents to keep the public API clean.

### 3. Types & Safety
- **Strong Typing:** Prefer specific types over primitives (e.g., use `Uuid` instead of `String` for IDs).
- **Ownership:** Leverage Rust's ownership model. Use `Arc<T>` for shared state/ports in services.
- **Async:** Use `tokio` for async operations, especially in the `inbound` (Axum) layer.

### 4. Error Handling
- **Domain Errors:** Define specific enums for domain-level errors (e.g., `FileManagerError`).
- **Result Type:** Always return `Result<T, E>` for operations that can fail.
- **Error Mapping:** Map lower-level errors (like `std::io::Error`) to domain errors at the boundary.
- **Panic:** Never use `unwrap()` or `expect()` in production code. Use `?` or handle the error.

### 5. Formatting
- Use standard `rustfmt` defaults. Run `cargo fmt` before committing.
- Maximum line length is generally 100 characters.

## 🧪 Testing Guidelines

- **Unit Tests:** Place unit tests in a `tests` module at the bottom of the file being tested.
- **Integration Tests:** Place in the `tests/` directory at the root (if applicable).
- **Mocking:** Implement traits (Ports) with mock structures for testing domain logic.
- **Documentation Tests:** Use doc-tests for simple examples in public APIs.

## 🤖 Interaction Rules for Agents

- **Read First:** Always read the corresponding Port (trait) and existing implementation before modifying an adapter.
- **Respect Boundaries:** Do not leak infrastructure details (like Axum types or IO errors) into the `domain` layer.
- **Verify:** Run `cargo check` and `cargo clippy` after every modification to ensure type safety and idiomaticity.
- **Tests:** When adding features, add corresponding tests in the `tests` module.
