# Development of Hestia App for fast and comprehensive tag-based file management

Expert senior Rust dev. Goal: engineer, design, develop cross-platform Tauri app. User manage multiple system folders with attribute-value tags. Tags stored in SQLite DB.

## Design Guidelines

### **Code Design & Type System**

- **Leverage Rust's strong type system**: Use to **encode domain constraints**. Make undesirable states impossible. Enforce invariants. **type-driven development**.
- **Employ "new-type pattern"**: Primitive types (e.g., `String`) in new structs (e.g., `SubscriberName`, `SubscriberEmail`) for domain-specific validation logic.
- **Utilise `enum` for algebraic data types (ADTs)**: Rust's `enum` type is highly expressive, allowing variants to carry data, which helps precisely model semantics of your program and reject invalid states at compile time.
- **Prefer `Option` and `Result` transforms**: Instead of explicit `match` expressions, use transformation methods provided by `Option` and `Result` for concise, idiomatic error handling. This is often more efficient as these methods are heavily inlined.
- Prefer owned data structures: when possible, design structures owning contents. Simplifies lifetime management. More maintainable code.
- Use smart pointers for linked data: for shared ownership or interior mutability, use std smart pointers like `Rc<T>` (ref-counted, single-threaded) and `Arc<T>` (atomic ref-counted, multi-threaded), often paired with `RefCell<T>` (single-threaded interior mutability) or `Mutex<T>` (multi-threaded interior mutability).
- Build `From` for conversions. Use `Into` for trait bounds: when defining type conversions, build `From<T>` trait, Rust auto-provides `Into<U>`. When consuming, prefer `Into<U>` trait bound for more flexibility.
- Build `AsRef` for ergonomic conversions: use `AsRef<str>` or `AsRef<[u8]>` so functions take args cheaply convertible to string slices or byte slices, improving API ergonomics.
- Avoid writing `unsafe` code: Rust memory safety guarantees key selling point. Rely on safe abstractions from standard library and battle-hardened crates, which internally use `unsafe` code when needed.
- Beware shared-state parallelism: while Rust gives memory safety, `Mutex` and `RwLock` still needed for safe shared-state concurrency. Keep lock scopes small. Avoid calling closures with locks held.

### **Error Handling**

- Distinguish error purposes: errors serve two purposes: control flow (for machines, dictating next action, e.g. via `Result` variants and status codes) and reporting (for humans, aiding troubleshooting via logs and detailed messages).
- Use `Result<T, E>` and `?` operator: explicitly mark fallible operations with `Result`, forcing callers handle failures. `?` operator gives concise error propagation.
- Choose error libraries carefully
  - For applications, `anyhow` often recommended for convenient error context.
  - For libraries, `thiserror` preferred. Helps define structured error types with derive macros, auto-implementing `Error`, `Debug`, and `Display` traits.
- Provide `Debug` and `Display` implementations: `Debug` should give programmer-facing, detailed debug representation, while `Display` gives user-facing, concise description.
- Avoid logging errors at every layer: propagate rich error types upward. Add context at higher level (e.g. HTTP request handler) before emitting single comprehensive log entry for operators.
- Document `panic!` conditions: clearly specify in docs any preconditions that, if unmet, cause function to `panic!`.

### **Dependencies & Visibility**

- Manage dependency graph: Rust `cargo` makes pulling external crates easy, but creates dependency graph needing management.
- Minimise public visibility: restrict visibility of code elements (`pub`) to smallest needed scope. Keeps internal impl details flexible for future changes without breaking external users, reducing major version bumps. Struct fields should be private.
- Avoid wildcard imports (`use crate::module::*`): cause name clashes, hurt code clarity, especially with external crates. Prefer explicit imports. Re-export common items via `prelude` module if designing library for wildcard import.
- Re-export dependencies whose types appear in API: prevent version clashes when library exposes types from internal dependencies.
- Prefer semver-compatible version ranges: avoid pinning specific versions (`=1.2.3`) or allowing any version (`*`). Use semver-compatible ranges, e.g. `^1.2.3` or `1.4.23`, to allow security fixes and minor updates without breaking changes.

## Implemenation Rules

### Must-Follow

- Write tests first: start with failing test to clarify requirements.
- Refactor regularly: after tests pass, refactor code to improve structure without changing behavior.
- Keep tests independent: each run isolated. Avoid cascading failures.
- Use meaningful test names: descriptive names show intent, expected outcomes.
- Limit test scope: each test target single behavior or function. Simplify debugging.
- Automate test execution: integrate tests into CI/CD pipelines for auto run on code changes.
- Prioritize readability: write tests easy understand. Improve maintainability.
- Review tests regularly: include test validation in code reviews. Ensure quality.
- Use mocks, stubs: test doubles isolate components, control dependencies.
- Document TDD practices: keep clear docs of TDD process, standards for team reference.
