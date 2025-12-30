# Rust Backend Skills Transformation - Context Document

**Last Updated:** 2025-11-02 17:30:00

---

## 🚀 Quick Handoff for Next Session

**Current Status**: Phase 1.2 COMPLETE ✅ - Ready for Task 1.3

**What to Do Next**:
1. Create new file: `.claude/skills/backend-dev-guidelines/resources/error-handling.md`
2. Use Hestia references: `src-tauri/src/errors/db.rs`, `src-tauri/src/errors/app.rs`
3. Follow structure from Task 1.3 in tasks document (lines 82-114)
4. Include 6+ complete error type examples with thiserror, anyhow, Serialize

**Critical Context**:
- SKILL.md is 100% Rust now (with user's Controller layer updates)
- User updated SKILL.md with important changes (8 core principles, anyhow::Result preference)
- tauri-commands.md created with 5+ examples from Hestia
- 11 old TypeScript resource files still exist (need replacement)
- Navigation links in SKILL.md point to new resource file names
- All Phase 1 tasks (1.3, 1.4) are P0 BLOCKING

**Files Completed So Far**:
- ✅ `.claude/skills/backend-dev-guidelines/SKILL.md` (COMPLETE)
- ✅ `.claude/skills/backend-dev-guidelines/resources/tauri-commands.md` (COMPLETE)

**Files to Create Next**:
- ⬜ `resources/error-handling.md` (Task 1.3 - NEXT)
- ⬜ `resources/seaorm-database.md` (Task 1.4)

---

## Current Session Progress (2025-11-02)

### ✅ COMPLETED: Task 1.1 - Update SKILL.md (Main File)

**Status**: COMPLETE

### ✅ COMPLETED: Task 1.2 - Create tauri-commands.md

**Status**: COMPLETE - Ready for Task 1.3

**What Was Done**:
- Created comprehensive Tauri command handlers guide (~750 lines)
- All examples taken from Hestia's working codebase
- Observed user's architecture updates (Controller layer, 8 principles)
- Documented State injection, Result returns, async patterns
- 5+ complete examples from tag_management.rs
- Best practices and anti-patterns sections
- Testing patterns included

**Key Implementation Details**:
1. Overview section with architectural role (including Controller layer)
2. Basic command patterns with #[command] attribute
3. State injection with lock scope best practices
4. Result return types: anyhow::Result (internal) vs Result<T, E> (IPC)
5. Async command patterns (basic, transactions, background tasks)
6. Command registration in main.rs
7. 5 complete examples:
   - Create tag (simple CRUD)
   - Get all tags (query with filter)
   - Search tags (pattern matching)
   - Add tag to file (relationship management)
   - Delete tag (transaction with cascade)
8. Best practices (✅ DO / ❌ DON'T)
9. Testing section with tokio::test

**Validation**:
- All code examples from working Hestia codebase
- Observes user's Controller layer architecture
- Shows both anyhow::Result and Result<T, E> patterns
- Emphasizes minimal lock scopes
- Comprehensive error handling

**Key Implementation Details**:
1. Header metadata updated to describe Rust/Tauri/SeaORM patterns
2. Purpose section rewritten for Rust backend in Tauri apps
3. When to Use section lists: Tauri commands, SeaORM, tokio, thiserror, state management
4. Quick Start checklists replaced with Tauri-specific workflows
5. Architecture diagram updated: Frontend → Tauri IPC → Command → Service → Operations → DB
6. Directory structure matches Hestia's `src-tauri/src/` exactly
7. All 7 Core Principles rewritten for Rust idioms
8. Common Imports replaced with Rust crates (tauri, sea-orm, thiserror, anyhow, tokio, serde, tracing)
9. Navigation Guide updated with new resource file paths
10. Resource Files section updated with 10 new Rust-focused files

**Validation Results**:
- ✅ No TypeScript/JavaScript references (grep verified)
- ✅ All code examples use valid Rust syntax
- ✅ Directory structure matches Hestia
- ✅ Links updated to new resource files (resources/*.md)
- ✅ Maintains progressive disclosure structure

**Files Modified/Created This Session**:
1. `.claude/skills/backend-dev-guidelines/SKILL.md` (MODIFIED - Session 1)
   - Location: `/home/emmi/projects/projects/hestia_tauri/.claude/skills/backend-dev-guidelines/SKILL.md`
   - Before: 290 lines, 100% TypeScript
   - After: 348 lines, 100% Rust (user made additional refinements)
   - Changes: 10 major sections completely rewritten
   - User Updates: Added Controller layer, changed to 8 principles, anyhow::Result preference

2. `.claude/skills/backend-dev-guidelines/resources/tauri-commands.md` (CREATED - Session 2)
   - Location: `/home/emmi/projects/projects/hestia_tauri/.claude/skills/backend-dev-guidelines/resources/tauri-commands.md`
   - Size: ~750 lines
   - Content: Complete Tauri command patterns with 5+ examples
   - References: tag_management.rs, app.rs from Hestia

**Next Immediate Steps**:
1. Proceed to Task 1.3: Create `resources/error-handling.md`
2. Location: `.claude/skills/backend-dev-guidelines/resources/error-handling.md`
3. Document thiserror enum patterns with all variant types
4. Show anyhow::Context for error chaining
5. Include examples from Hestia's `errors/db.rs`, `errors/app.rs`
6. Cover: error philosophy, defining enums, conversions, context, Tauri serialization, propagation
7. Target: L effort (4-5 hours), 6+ complete error type examples

**Important Notes for Next Session**:
- SKILL.md footer shows: "Phase 1.1 COMPLETE ✅"
- All navigation links now point to `resources/*.md` files
- Old TypeScript resource files STILL EXIST (11 files) - need to be replaced/deleted
- Resource files directory: `.claude/skills/backend-dev-guidelines/resources/`
- Current files are 100% TypeScript/Express/Prisma patterns
- Need to replace/create 10 new Rust resource files in Phase 1-4
- Task 1.2, 1.3, 1.4 are all P0 BLOCKING tasks

**Old Files to Replace/Delete**:
1. ❌ architecture-overview.md (merge into SKILL.md - DONE)
2. ❌ routing-and-controllers.md → REPLACE with tauri-commands.md
3. ❌ async-and-errors.md → SPLIT into error-handling.md + async-patterns.md
4. ❌ testing-guide.md → REPLACE (complete rewrite)
5. ❌ database-patterns.md → REPLACE with seaorm-database.md
6. ❌ configuration.md → MERGE into state-management.md
7. ❌ validation-patterns.md → MERGE into type-driven-design.md
8. ❌ middleware-guide.md → DELETE (no Tauri equivalent)
9. ❌ sentry-and-monitoring.md → REPLACE with tracing-logging.md
10. ❌ services-and-repositories.md → MERGE into seaorm-database.md
11. ❌ complete-examples.md → REPLACE (all new Rust examples)

---

## Project Context

### What We're Doing
Transforming the `.claude/skills/backend-dev-guidelines` skill from TypeScript/Express/Prisma patterns to Rust/Tauri/SeaORM best practices, ensuring alignment with:
1. The actual Hestia project implementation
2. Official Rust documentation and best practices
3. CLAUDE.md design guidelines
4. Tauri-specific patterns

### Why We're Doing This
The current backend skill contains 100% TypeScript patterns (Express routes, Prisma queries, Jest tests) which are completely inapplicable to the Rust/Tauri Hestia application. This misalignment causes confusion and provides no value to developers working on Hestia.

### Success Criteria
- Zero TypeScript references
- All code examples compile
- 100% alignment with Hestia patterns
- Covers all CLAUDE.md guidelines
- Maintains progressive disclosure structure

---

## Key Files and Locations

### Skill Files (To Be Modified)

**Main Skill File**:
- Location: `.claude/skills/backend-dev-guidelines/SKILL.md`
- Lines: 290
- Status: Needs complete transformation
- Priority: P0 - CRITICAL PATH

**Resource Files Directory**:
- Location: `.claude/skills/backend-dev-guidelines/resources/`
- Current count: 11 files
- Target count: 10 files (1 merge, 3 new)

### Hestia Reference Files

**Command Handlers** (Primary reference for Task 1.2):
```
src-tauri/src/commands/
├── tag_management.rs          # Tag CRUD operations
│   └── create_tag, get_all_tags, update_tag, delete_tag
├── watched_folder_management.rs  # Folder management
│   └── add_watched_folder, get_all_watched_folders, remove_watched_folder
└── filter_management.rs       # Filter operations
    └── create_filter, get_filters, apply_filter
```

**Key Patterns in Hestia Commands**:
- All use `#[command]` macro
- State injection: `State<'_, Mutex<AppState>>`
- Return type: `Result<T, AppError>` or `Result<T, DbError>`
- Async: Most are `async fn`
- Serialization: All return types implement `Serialize`
- Error handling: Use `?` operator for propagation

**Database Operations** (Primary reference):
```
src-tauri/src/database/
├── operations.rs              # FileOperations, ThumbnailOperations
├── manager.rs                 # DatabaseManager, connection management
└── models.rs                  # Database models
```

**Error Handling** (Primary reference):
```
src-tauri/src/errors/
├── db.rs                      # DbError enum with thiserror
├── app.rs                     # AppError enum with thiserror
└── file_system.rs             # FileSystemError enum
```

**State Management** (Primary reference):
```
src-tauri/src/config/
├── app.rs                     # AppState struct, Arc<DatabaseManager>
└── database.rs                # DatabaseSettings, SqliteConfig
```

**Testing** (Primary reference):
```
src-tauri/src/tests/
└── watcher.rs                 # tokio::test examples, tempdir usage
```

**Async Patterns** (Primary reference):
```
src-tauri/src/file_system/
├── watcher.rs                 # FileWatcherHandler, async channels
└── scanner.rs                 # DirectoryScanner, async operations
```

### Documentation References

**CLAUDE.md** (Project guidelines):
- Location: `app/CLAUDE.md`
- Key sections:
  - Design Guidelines (lines 17-30): Type system, ownership, error handling
  - Error Handling (lines 32-41): Result<T, E>, thiserror, anyhow
  - Implementation Rules (lines 50-64): TDD, testing requirements

**Context7 Documentation** (Already fetched):
- Rust Book: Error handling, ownership, type system
- Tokio docs: async/await, spawning, runtime
- SeaORM docs: Entity, ActiveModel, transactions
- Tauri docs: Commands, state management, Rust backend

---

## Critical Decisions Made

### Decision 1: File Structure
**Decision**: Transform existing files rather than delete and recreate
**Rationale**:
- Maintains git history
- Preserves file structure
- Easier to track changes

**Impact**: Each file will be completely rewritten but keep same filename where applicable

### Decision 2: Architecture Model
**Decision**: Simplify from 4-layer to 3-layer architecture
**TypeScript Model**: Routes → Controllers → Services → Repositories → Database
**Rust Model**: Commands → Services → Operations → Database

**Rationale**:
- Tauri commands already provide request handling (no separate controller layer needed)
- Rust's type system reduces need for validation layer
- Simpler architecture better matches Hestia's actual implementation

**Impact**: Fewer concepts to learn, clearer separation of concerns

### Decision 3: Resource File Changes
**Decision**: Create 10 new resource files with 3 entirely new topics

**New Files**:
1. `async-patterns.md` - tokio-specific patterns
2. `type-driven-design.md` - Rust type system emphasis
3. `ownership-patterns.md` - Rust ownership/borrowing

**Merged Files**:
- `configuration.md` → merge into `state-management.md`
- `validation-patterns.md` → merge into `type-driven-design.md`
- `services-and-repositories.md` → merge into `seaorm-database.md`

**Removed Files**:
- `middleware-guide.md` - No direct Tauri equivalent (use state/commands instead)

**Rationale**: Better reflects Rust/Tauri development patterns and emphasizes Rust-specific concerns

### Decision 4: Example Source
**Decision**: All code examples drawn from actual Hestia codebase
**Rationale**:
- Guaranteed to compile
- Reflects real-world usage
- Shows actual project patterns
- Easier to maintain

**Impact**: Examples will reference actual file paths in Hestia for traceability

### Decision 5: Progressive Disclosure
**Decision**: Maintain existing progressive disclosure structure
**Format**: Brief overview in SKILL.md → Detailed patterns in resource files

**Rationale**:
- Proven effective structure
- Prevents overwhelming users
- Allows quick reference vs deep learning

**Impact**: SKILL.md stays under 500 lines, resources provide depth

---

## Dependencies and Constraints

### Technical Dependencies

**Rust Toolchain**:
- Required for compiling test examples
- Version: Match Hestia's rust-toolchain.toml
- No additional setup needed (already available)

**Hestia Codebase**:
- Must be current and building
- Reference for all patterns
- Source of truth for examples

**Documentation Sources**:
- Context7 for official docs (already fetched)
- Hestia codebase for examples
- CLAUDE.md for guidelines

### Process Constraints

**Sequential Phases**:
- Phase 1 must complete before Phase 2
- Phase 2 must complete before Phase 3
- Phases 1-3 must complete before Phase 4
- Phase 5 validates everything

**Quality Gates**:
- All code must compile (Task 5.2)
- All links must work (Task 5.1)
- CLAUDE.md alignment verified (Task 5.3)

**Time Constraints**:
- Estimated 54-66 hours total
- Critical path: 20-26 hours
- Recommend 3-week timeline

---

## Pattern Reference Quick Guide

### Command Handler Pattern
```rust
#[command]
pub async fn operation_name(
    state: State<'_, Mutex<AppState>>,
    param: Type,
) -> Result<ReturnType, ErrorType> {
    // Implementation
}
```

**Where to document**: `tauri-commands.md`

### Error Definition Pattern
```rust
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum DomainError {
    #[error("Description: {field}")]
    VariantName { field: Type },

    #[error("Simple error")]
    SimpleVariant,
}
```

**Where to document**: `error-handling.md`

### SeaORM Query Pattern
```rust
let result = Entity::find()
    .filter(Column::Field.eq(value))
    .one(&*connection)
    .await?;
```

**Where to document**: `seaorm-database.md`

### State Management Pattern
```rust
pub struct AppState {
    pub database: Arc<DatabaseManager>,
    pub cache: Arc<RwLock<HashMap<K, V>>>,
}
```

**Where to document**: `state-management.md`

### Async Pattern
```rust
tokio::spawn(async move {
    // Background task
});
```

**Where to document**: `async-patterns.md`

### Test Pattern
```rust
#[tokio::test]
async fn test_operation() -> Result<()> {
    // Test implementation
    Ok(())
}
```

**Where to document**: `testing-guide.md`

---

## Common Pitfalls to Avoid

### Pitfall 1: Including Uncompilable Code
**Problem**: Code examples that look correct but don't compile
**Solution**: Test every code block in a separate Rust project
**Detection**: Compilation check phase (Task 5.2)

### Pitfall 2: Mixing Paradigms
**Problem**: Showing TypeScript patterns alongside Rust
**Solution**: Complete removal of all TypeScript references
**Detection**: Text search for TypeScript keywords

### Pitfall 3: Missing Hestia Patterns
**Problem**: Documenting theoretical patterns not used in Hestia
**Solution**: Only document patterns actually present in Hestia codebase
**Detection**: Cross-reference with Hestia source files

### Pitfall 4: Overcomplicated Examples
**Problem**: Examples too complex to understand quickly
**Solution**: Start simple, build complexity gradually
**Detection**: User feedback during review

### Pitfall 5: Broken Links
**Problem**: References between files don't work
**Solution**: Validate all links in Task 5.1
**Detection**: Link checking phase

---

## Testing Strategy

### Code Example Testing

**Approach**: Create a separate Rust project with all dependencies

**Setup**:
```bash
cd /tmp
cargo new rust-backend-skills-test
cd rust-backend-skills-test
```

**Dependencies** (add to Cargo.toml):
```toml
[dependencies]
tauri = { version = "2", features = ["test"] }
sea-orm = { version = "1.1.10", features = ["sqlx-sqlite", "runtime-tokio-rustls"] }
tokio = { version = "*", features = ["full"] }
thiserror = "2.0"
anyhow = "1.0"
tracing = "0.1"
serde = { version = "1", features = ["derive"] }
```

**Test Process**:
1. Extract code example
2. Place in test project
3. Run `cargo check` or `cargo test`
4. Fix any issues
5. Update skill documentation

### Progressive Testing

**Phase 1**: Test as you write (immediate feedback)
**Phase 5**: Comprehensive test (all examples at once)

---

## Resource Estimation Details

### Time Breakdown by Activity

**Writing** (60%): ~35-40 hours
- Drafting content
- Creating examples
- Structuring information

**Research** (20%): ~12-14 hours
- Analyzing Hestia code
- Cross-referencing documentation
- Validating patterns

**Testing** (15%): ~8-10 hours
- Compiling examples
- Running tests
- Fixing issues

**Review** (5%): ~3-4 hours
- Cross-referencing
- Link validation
- Final checks

### Effort Level Definitions

- **S (Small)**: 1-2 hours - Simple updates, link checks
- **M (Medium)**: 3-4 hours - Single resource file, focused content
- **L (Large)**: 4-5 hours - Complex resource file, multiple patterns
- **XL (Extra Large)**: 6-8 hours - Comprehensive content, many examples

---

## Success Validation Checklist

### Content Quality
- [ ] All code examples compile without errors
- [ ] Examples drawn from actual Hestia code
- [ ] Explanations clear and concise
- [ ] Progressive disclosure maintained
- [ ] Consistent terminology throughout

### Technical Accuracy
- [ ] Patterns match Hestia implementation
- [ ] Aligns with Rust best practices
- [ ] Follows CLAUDE.md guidelines
- [ ] tokio patterns correct
- [ ] SeaORM usage correct
- [ ] Tauri patterns correct

### Completeness
- [ ] All 10 resource files created
- [ ] SKILL.md fully transformed
- [ ] All navigation links work
- [ ] No TypeScript references remain
- [ ] All phases complete

### Usability
- [ ] Examples are copy-paste ready
- [ ] Clear "why" explanations
- [ ] Common pitfalls documented
- [ ] Anti-patterns called out
- [ ] Related files cross-referenced

---

## Communication Plan

### Status Updates
- **Frequency**: After each phase completion
- **Format**: Phase summary with completed tasks
- **Content**: What was done, any issues, next steps

### Decision Points
- **Architecture changes**: Consult before finalizing
- **Pattern interpretation**: Verify against Hestia code
- **Scope questions**: Clarify if uncertain

### Review Checkpoints
- **Phase 1 complete**: Review critical path files
- **Phase 4 complete**: Review complete examples
- **Phase 5 complete**: Final validation

---

## Future Enhancements (Out of Scope)

These items are explicitly out of scope for this transformation but documented for future consideration:

1. **Advanced SeaORM Patterns**
   - Custom derive macros
   - Complex relationship modeling
   - Performance optimization techniques

2. **Tauri IPC Optimization**
   - Streaming data patterns
   - Binary data handling
   - Custom invoke handlers

3. **Cross-Platform Considerations**
   - Platform-specific code
   - Conditional compilation
   - Platform testing strategies

4. **Advanced Testing**
   - Property-based testing
   - Benchmark testing
   - Integration test suites

5. **Deployment Patterns**
   - CI/CD for Tauri apps
   - Auto-update strategies
   - Release management

---

## Glossary of Terms

**ActiveModel**: SeaORM's representation for insert/update operations, wraps fields in ActiveValue

**ADT (Algebraic Data Type)**: Rust enum with variants that carry data

**Arc**: Atomic Reference Counted pointer for thread-safe shared ownership

**Command Handler**: Tauri function marked with #[command] that frontend can invoke

**Entity**: SeaORM's representation of a database table

**Mutex**: Mutual exclusion lock for interior mutability

**Newtype Pattern**: Wrapping a primitive type in a struct for type safety

**Progressive Disclosure**: UX pattern showing overview first, details on demand

**Result<T, E>**: Rust's error handling type with Ok(T) or Err(E) variants

**RwLock**: Reader-writer lock allowing multiple readers or one writer

**State**: Tauri's managed application state accessible in commands

**thiserror**: Rust crate for deriving Error trait implementations

**tokio**: Async runtime for Rust

---

**End of Context Document**

This document provides all the context needed to understand and execute the transformation plan. It should be referenced throughout the implementation to ensure consistency and completeness.
