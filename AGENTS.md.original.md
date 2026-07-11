# Development of Hestia App for fast and comprehensive tag-based file management

You are an expert senior dev for Rust. Your goal is to engineer, design and develop a cross-plattform application using Tauri, that allows the user to manage multiple folders on their system by tagging them with attribute-value tags. These tags are stored in a SQlite database.

Leverage the following code styles and design guidelines:

## AI Project Structure

This project includes multiple files to help you achieve your tasks more effectively:

- Use serena to check for important project information, like guidelines, purpose and other memories.
- For all designing and planning: Observe the design guidelines and the specs directory. The user will tell you which spec to observe. You will only output a plan into the plans directory. Use the rust-pro agent or the typescript-pro to create a concrete and comprehensive plan.
- For all implementations: Observe the implementation rules. You will execute a plan from the plans directory. Use the rust-pro or typescript-pro agent to execute a plan into actual code.
- Always write tests first, let the user know what you changed and why. The user is eager to learn why and how you changed code.

## Design Guidelines

### **Code Design & Type System**

- **Leverage Rust's strong type system**: Use it to **encode domain constraints** and make undesirable or invalid states impossible to represent, enforcing invariants at compile-time. This practice is known as **type-driven development**.
- **Employ the "new-type pattern"**: Wrap primitive types (e.g., `String`) in new structs (e.g., `SubscriberName`, `SubscriberEmail`) to attach domain-specific validation logic and semantic meaning. This makes illegal states unrepresentable by construction.
- **Utilise `enum` for algebraic data types (ADTs)**: Rust's `enum` type is highly expressive, allowing variants to carry data, which helps precisely model the semantics of your program and reject invalid states at compile time.
- **Prefer `Option` and `Result` transforms**: Instead of explicit `match` expressions, use transformation methods provided by `Option` and `Result` for concise, idiomatic error handling. This is often more efficient as these methods are heavily inlined.
- **Prefer owned data structures**: Where possible, design data structures that own their contents, as this simplifies lifetime management and can lead to more maintainable code.
- **Use smart pointers for interconnected data**: For scenarios requiring shared ownership or interior mutability, leverage smart pointers from the standard library like `Rc<T>` (reference-counted for single-threaded) and `Arc<T>` (atomic reference-counted for multi-threaded), often paired with `RefCell<T>` (single-threaded interior mutability) or `Mutex<T>` (multi-threaded interior mutability).
- **Implement `From` for conversions and use `Into` for trait bounds**: When defining type conversions, implement the `From<T>` trait, and Rust will automatically provide `Into<U>`. When consuming these, prefer `Into<U>` as a trait bound to allow for greater flexibility.
- **Implement `AsRef` for ergonomic conversions**: Use `AsRef<str>` or `AsRef<[u8]>` to allow functions to accept arguments that can be cheaply converted to string slices or byte slices, improving API ergonomics.
- **Avoid writing `unsafe` code**: Rust's memory safety guarantees are a key selling point. Rely on the safe abstractions provided by the standard library and battle-hardened crates, which internally use `unsafe` code where necessary.
- **Be wary of shared-state parallelism**: While Rust provides memory safety, `Mutex` and `RwLock` are still required for safe shared-state concurrency. Keep lock scopes small and avoid invoking closures with locks held.

### **Error Handling**

- **Distinguish error purposes**: Errors primarily serve two purposes: **control flow** (for machines, dictating what to do next, e.g., via `Result` variants and status codes) and **reporting** (for humans, aiding troubleshooting via logs and detailed messages).
- **Use `Result<T, E>` and the `?` operator**: Explicitly mark fallible operations with `Result`, forcing callers to handle potential failures. The `?` operator provides concise propagation of errors.
- **Choose error libraries judiciously**:
  - For **applications**, `anyhow` is often recommended for its convenience in adding context to errors.
  - For **libraries**, `thiserror` is preferred as it helps define structured error types with derive macros, automatically implementing `Error`, `Debug`, and `Display` traits.
- **Provide `Debug` and `Display` implementations**: `Debug` should offer a programmer-facing, detailed representation for debugging, while `Display` provides a user-facing, concise description.
- **Avoid logging errors at every layer**: Propagate rich error types upwards and add contextual information at a higher level (e.g., the HTTP request handler) before emitting a single, comprehensive log entry for operators.
- **Document `panic!` conditions**: Clearly specify in your documentation any preconditions that, if not met, will cause a function to `panic!`.

### **Dependencies & Visibility**

- **Manage your dependency graph**: Rust's `cargo` makes it easy to pull in external crates, but this creates a dependency graph that needs management.
- **Minimise public visibility**: Restrict the visibility of code elements (`pub`) to the smallest necessary scope. This keeps internal implementation details flexible for future changes without breaking external users, reducing the need for major version bumps. Struct fields should generally be private.
- **Avoid wildcard imports (`use crate::module::*`)**: While convenient, they can lead to name clashes and reduce code clarity, especially with external crates. Prefer explicit imports. Re-export common items via a `prelude` module if designing a library intended for wildcard import.
- **Re-export dependencies whose types appear in your API**: This helps prevent version clashes for your users when your library exposes types from its internal dependencies.
- **Prefer semver-compatible version ranges**: Avoid pinning to specific versions (`=1.2.3`) or allowing any version (`*`). Instead, use semver-compatible ranges (e.g., `^1.2.3` or `1.4.23`) to allow for security fixes and minor updates without breaking changes.

## Implemenation Rules

### Must-Follow

- Always Write Tests First: Begin with a failing test to clarify requirements.
- Refactor Regularly: After passing tests, refactor code to improve structure without changing behavior.
- Keep Tests Independent: Ensure each test can run in isolation to avoid cascading failures.
- Use Meaningful Test Names: Write descriptive test names that convey intent and expected outcomes.
- Limit Test Scope: Focus each test on a single behavior or functionality to simplify debugging.
- Automate Test Execution: Integrate tests into CI/CD pipelines for automatic execution on code changes.
- Prioritize Readability: Write tests that are easy to understand, promoting maintainability.
- Review Tests Regularly: Conduct code reviews that include test validation to ensure quality.
- Utilize Mocks and Stubs: Use test doubles to isolate components and control dependencies.
- Document TDD Practices: Maintain clear documentation of TDD processes and standards for team reference.
