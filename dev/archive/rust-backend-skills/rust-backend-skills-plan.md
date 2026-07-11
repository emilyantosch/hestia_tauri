# Rust Backend Skills Transformation - Comprehensive Strategic Plan

**Last Updated:** 2025-11-02

---

## Executive Summary

### Overview
Transform the `backend-dev-guidelines` skill from TypeScript/Express/Prisma patterns to Rust/Tauri/SeaORM best practices, aligning with the actual Hestia project implementation and official Rust documentation.

### Current State
- **100% TypeScript-focused**: All 11 resource files contain Express, Prisma, Jest, and Node.js patterns
- **Misaligned with Hestia**: The Hestia project is a Rust/Tauri application using SeaORM, tokio, and thiserror
- **Outdated patterns**: BaseController inheritance, Express middleware, Zod validation—none applicable to Rust
- **290 lines of TypeScript guidance** that needs complete replacement

### Target State
- **100% Rust-focused**: All patterns aligned with Rust idioms and Tauri architecture
- **Hestia-aligned**: Examples drawn directly from actual Hestia implementation
- **Best practices**: Aligned with official Rust documentation, tokio patterns, SeaORM practices
- **Production-ready**: Copy-paste examples that work in real Tauri applications

### Business Value
1. **Reduced Development Time**: Developers can reference correct patterns immediately
2. **Code Quality**: Enforces Rust best practices and type-driven development
3. **Consistency**: All backend code follows same architectural patterns
4. **Onboarding**: New developers can learn from accurate, working examples
5. **Maintainability**: Clear separation of concerns and testable architecture

### Success Metrics
- ✅ Zero TypeScript references remaining
- ✅ All code examples compile and run
- ✅ 100% alignment with Hestia project patterns
- ✅ All CLAUDE.md guidelines represented
- ✅ Progressive disclosure maintained (brief overview → detailed resources)

---

## Current State Analysis

### Skill Structure Analysis

**Location**: `.claude/skills/backend-dev-guidelines/`

**Main File**: `SKILL.md` (290 lines)
- **Purpose**: Comprehensive backend development guide
- **Tech Stack**: Express, Prisma, Zod, Jest, TypeScript
- **Structure**: 7 core principles, 11 resource files
- **Status**: Complete but 100% misaligned with Rust

**Resource Files** (11 total):
1. `architecture-overview.md` - Layered architecture (Routes → Controllers → Services → Repositories)
2. `routing-and-controllers.md` - Express routing, BaseController pattern
3. `async-and-errors.md` - Promise chains, try-catch, custom error types
4. `testing-guide.md` - Jest testing, mocking PrismaService
5. `database-patterns.md` - Prisma operations, transactions, N+1 queries
6. `configuration.md` - UnifiedConfig pattern, process.env usage
7. `validation-patterns.md` - Zod schemas
8. `middleware-guide.md` - Express middleware
9. `sentry-and-monitoring.md` - Sentry integration
10. `services-and-repositories.md` - Service/repo patterns
11. `complete-examples.md` - Full implementation examples

### Gap Analysis

| TypeScript Pattern | Rust Equivalent | Status |
|-------------------|-----------------|---------|
| Express routes | Tauri commands | ❌ Not documented |
| BaseController class | Command handler functions | ❌ Not documented |
| Prisma queries | SeaORM operations | ❌ Not documented |
| try-catch blocks | Result<T, E> + ? operator | ❌ Not documented |
| Jest tests | Cargo test with tokio::test | ❌ Not documented |
| Zod validation | Manual validation or custom | ❌ Not documented |
| Express middleware | Tauri state/middleware | ❌ Not documented |
| process.env | Config structs | ❌ Not documented |
| Sentry | tracing crate | ❌ Not documented |
| Promise chains | async/await with tokio | ❌ Not documented |

### Hestia Project Patterns (Actual Implementation)

**Technology Stack**:
- **Framework**: Tauri v2
- **Database**: SeaORM 1.1.10 with SQLite
- **Async Runtime**: tokio
- **Error Handling**: thiserror + anyhow
- **Testing**: Cargo test with tokio::test
- **Logging**: tracing + tracing-subscriber
- **State Management**: Arc/Mutex

**Architecture Pattern**:
```
Tauri Command Handler
    ↓
Service Layer (business logic)
    ↓
Operations/Repository (database access)
    ↓
SeaORM (SQLite)
```

**Key Patterns Observed**:
1. **Commands**: `#[command]` macro, State injection, Result<T, E> returns
2. **Errors**: thiserror enums with Serialize for frontend
3. **Database**: SeaORM ActiveModel pattern, transactions with connection.begin()
4. **State**: Arc<DatabaseManager> for shared ownership, Mutex for mutability
5. **Testing**: #[tokio::test], tempdir for test isolation
6. **Async**: tokio::spawn, join!, timeout patterns

---

## Proposed Future State

### New Architecture Vision

**Skill Purpose**: Establish Rust/Tauri best practices for backend development

**Core Principles** (7 updated):
1. Commands only delegate (no business logic)
2. Use Result<T, E> for error handling
3. Leverage the type system (make illegal states unrepresentable)
4. Prefer Arc for shared ownership
5. Use thiserror for domain errors
6. Instrument with tracing
7. Write tests first (TDD with tokio::test)

**Layered Architecture**:
```
Tauri Command (#[command])
    ↓
Command Handler (validate, delegate)
    ↓
Service Layer (business logic)
    ↓
Operations/Repository (SeaORM)
    ↓
Database (SQLite)
```

### Resource Files Structure (10 new files)

1. **tauri-commands.md** - Command handlers, state management, serialization
2. **seaorm-database.md** - Entity usage, ActiveModel, transactions, queries
3. **error-handling.md** - thiserror enums, anyhow context, ? operator
4. **async-patterns.md** - tokio async/await, spawn, join/select
5. **state-management.md** - Arc/Mutex/RwLock, Tauri State
6. **testing-guide.md** - Cargo test, tokio::test, mocking, assertions
7. **tracing-logging.md** - tracing macros, #[instrument], structured logging
8. **type-driven-design.md** - Newtypes, enums, Option/Result, traits
9. **ownership-patterns.md** - Borrowing, lifetimes, Arc, Clone strategies
10. **complete-examples.md** - Full end-to-end examples from Hestia

**Files to Remove**:
- `middleware-guide.md` (Express-specific)
- `services-and-repositories.md` (merged into seaorm-database.md)
- `validation-patterns.md` (merged into type-driven-design.md)
- `sentry-and-monitoring.md` (replaced by tracing-logging.md)
- `configuration.md` (merged into state-management.md)

---

## Implementation Phases

### Phase 1: Foundation (Files 1-4) - CRITICAL PATH
**Estimated Effort**: XL (20-24 hours)
**Dependencies**: None
**Risk Level**: Low

Update the main SKILL.md file and create the four most critical resource files that establish the core patterns.

#### Task 1.1: Update SKILL.md (Main File)
**Effort**: L (4-5 hours)
**Priority**: P0 - Blocking

**Objectives**:
- Replace all TypeScript examples with Rust
- Update architecture diagram to show Tauri command flow
- Rewrite core principles for Rust
- Update directory structure to reflect Hestia's actual structure
- Replace common imports with Rust equivalents

**Acceptance Criteria**:
- [ ] Zero TypeScript/JavaScript references
- [ ] All code examples are valid Rust
- [ ] Directory structure matches Hestia's src-tauri/src/
- [ ] Common imports include: tauri, sea-orm, tokio, thiserror, anyhow, tracing
- [ ] Navigation guide links to new Rust resource files
- [ ] File naming conventions use snake_case

**Detailed Changes**:
```markdown
# Before (TypeScript)
```typescript
export class UserController extends BaseController {
    async getUser(req: Request, res: Response): Promise<void>
}
```

# After (Rust)
```rust
#[command]
pub async fn get_user(
    state: State<'_, Mutex<AppState>>,
    user_id: i32,
) -> Result<UserInfo, DomainError>
```
```

**Technical Notes**:
- Update lines 1-50: Purpose, When to Use, Quick Start
- Update lines 51-98: Architecture, Directory Structure
- Update lines 99-163: Core Principles (all 7)
- Update lines 164-189: Common Imports
- Update lines 224-241: Navigation Guide

#### Task 1.2: Create tauri-commands.md
**Effort**: L (4-5 hours)
**Priority**: P0 - Blocking

**Objectives**:
- Document Tauri command handler patterns
- Explain State injection with State<'_, Mutex<AppState>>
- Show Result<T, E> return types for fallible operations
- Document error serialization for frontend communication
- Provide async command examples with tokio
- Show command registration in tauri::generate_handler!

**Acceptance Criteria**:
- [ ] Covers all command patterns used in Hestia
- [ ] Examples compile without errors
- [ ] Includes state management patterns
- [ ] Documents both sync and async commands
- [ ] Shows error handling and serialization
- [ ] Includes at least 5 complete examples

**Content Structure**:
```markdown
# Tauri Command Handlers

## Overview
- What are Tauri commands
- When to use commands vs internal functions
- Command lifecycle

## Basic Command Pattern
- #[command] macro
- Simple functions
- Return types

## State Management in Commands
- State<'_, Mutex<AppState>>
- Locking patterns
- Avoiding deadlocks

## Error Handling
- Result<T, E> return types
- Serializable errors
- Frontend communication

## Async Commands
- tokio async patterns
- Spawning background tasks
- Timeout handling

## Complete Examples
- CRUD operation command
- File system command
- Database query command
- Error handling examples
- Background task command
```

**Examples from Hestia**:
- Reference: `src-tauri/src/commands/tag_management.rs`
- Pattern: `create_tag`, `get_all_tags`, `delete_tag`
- State pattern from `src-tauri/src/config/app.rs`

#### Task 1.3: Create error-handling.md
**Effort**: L (4-5 hours)
**Priority**: P0 - Blocking

**Objectives**:
- Document thiserror error enum patterns
- Show error variant types (unit, tuple, struct)
- Explain #[from] attribute for conversions
- Demonstrate anyhow::Context for error chaining
- Show custom Serialize implementation for Tauri
- Document ? operator usage
- Pattern matching on error types

**Acceptance Criteria**:
- [ ] Covers all error patterns in Hestia
- [ ] Shows thiserror vs anyhow usage
- [ ] Includes Serialize trait implementation
- [ ] Documents error propagation with ?
- [ ] Shows pattern matching examples
- [ ] Includes at least 6 complete error types

**Content Structure**:
```markdown
# Error Handling in Rust Backend

## Error Philosophy
- Errors for control flow vs reporting
- Result<T, E> as return type
- When to use thiserror vs anyhow

## Defining Error Enums
- Basic enum structure
- Unit variants
- Tuple variants
- Struct variants
- #[error(...)] attribute

## Error Conversions
- #[from] attribute
- Automatic From implementations
- Into trait usage

## Error Context
- anyhow::Context trait
- Adding context with .context()
- Error chains

## Tauri Error Serialization
- Serialize trait requirement
- Manual implementation
- Frontend error handling

## Error Propagation
- ? operator mechanics
- When to propagate vs handle
- Error boundaries

## Complete Examples
- DbError enum (from Hestia)
- AppError enum (from Hestia)
- Context wrapping examples
- Frontend serialization
```

**Examples from Hestia**:
- Reference: `src-tauri/src/errors/db.rs`, `src-tauri/src/errors/app.rs`
- Patterns: DbError, AppError, FileSystemError

#### Task 1.4: Create seaorm-database.md
**Effort**: XL (6-8 hours)
**Priority**: P0 - Blocking

**Objectives**:
- Document SeaORM Entity and ActiveModel patterns
- Show CRUD operations (create, read, update, delete)
- Explain transaction management
- Document query builders and filters
- Show relationship handling
- Demonstrate bulk operations
- Error handling with DbErr

**Acceptance Criteria**:
- [ ] Covers all SeaORM patterns in Hestia
- [ ] Includes Entity usage examples
- [ ] Documents ActiveModel pattern
- [ ] Shows transaction patterns
- [ ] Includes complex query examples
- [ ] Documents relationship handling
- [ ] At least 10 complete examples

**Content Structure**:
```markdown
# SeaORM Database Patterns

## SeaORM Overview
- Entity-based ORM
- ActiveModel pattern
- Query builder approach

## Entity Usage
- Finding records
- Filtering
- Selecting specific columns
- Ordering and pagination

## ActiveModel Pattern
- Creating new records
- Updating existing records
- Set vs NotSet
- Validation in ActiveModel

## Transactions
- Starting transactions
- Committing and rolling back
- Transaction scopes
- Error handling in transactions

## CRUD Operations
- Insert operations
- Select queries
- Update operations
- Delete operations
- Upsert patterns

## Relationships
- One-to-many
- Many-to-many
- Finding related records
- Eager loading

## Query Optimization
- N+1 query prevention
- Indexing strategies
- Query analysis

## Complete Examples
- FileOperations (from Hestia)
- TagOperations (from Hestia)
- Transaction examples
- Complex queries
```

**Examples from Hestia**:
- Reference: `src-tauri/src/database/operations.rs`
- Patterns: FileOperations, ThumbnailOperations, entity usage

---

### Phase 2: Async & State (Files 5-6) - HIGH PRIORITY
**Estimated Effort**: L (12-14 hours)
**Dependencies**: Phase 1 complete
**Risk Level**: Medium

#### Task 2.1: Create async-patterns.md
**Effort**: M (3-4 hours)
**Priority**: P1

**Objectives**:
- Document tokio async/await patterns
- Show tokio::spawn for background tasks
- Explain tokio::join! and tokio::select!
- Document tokio::time utilities (sleep, timeout)
- Async traits with async-trait crate
- Stream handling

**Acceptance Criteria**:
- [ ] Covers core tokio patterns
- [ ] Shows background task spawning
- [ ] Documents concurrent operations
- [ ] Includes timeout patterns
- [ ] Shows async trait usage
- [ ] At least 7 complete examples

**Content Structure**:
```markdown
# Async Patterns with Tokio

## Async Fundamentals
- async fn syntax
- .await operator
- Future trait basics

## Tokio Runtime
- Runtime creation
- Runtime handle
- Blocking operations

## Spawning Tasks
- tokio::spawn
- JoinHandle
- Task cancellation

## Concurrent Operations
- tokio::join!
- tokio::select!
- Racing futures

## Timeouts and Delays
- tokio::time::sleep
- tokio::time::timeout
- Interval timers

## Async Traits
- async-trait crate
- Trait bounds
- Send + 'static requirements

## Complete Examples
- Background processing
- Concurrent database queries
- Timeout handling
- Task coordination
```

**Examples from Hestia**:
- Reference: `src-tauri/src/file_system/watcher.rs`
- Patterns: FileWatcherHandler async operations

#### Task 2.2: Create state-management.md
**Effort**: L (4-5 hours)
**Priority**: P1

**Objectives**:
- Document Arc<T> for shared ownership
- Explain Mutex<T> vs RwLock<T>
- Show proper lock scope management
- Document Tauri State patterns
- Explain AppState structure
- Demonstrate state initialization

**Acceptance Criteria**:
- [ ] Covers Arc/Mutex/RwLock patterns
- [ ] Shows Tauri state management
- [ ] Documents lock scope best practices
- [ ] Includes deadlock prevention
- [ ] Shows AppState patterns
- [ ] At least 6 complete examples

**Content Structure**:
```markdown
# State Management in Tauri

## Shared Ownership
- Arc<T> for thread-safe reference counting
- Clone vs cloning the Arc
- When to use Arc

## Interior Mutability
- Mutex<T> for exclusive access
- RwLock<T> for read-heavy scenarios
- Lock scope management
- Deadlock prevention

## Tauri State
- app.manage() registration
- State<'_, T> injection
- Accessing state in commands
- AppHandle for global access

## AppState Pattern
- Designing AppState struct
- Initialization in setup
- Database connection sharing
- Cache management

## Complete Examples
- AppState from Hestia
- DatabaseManager with Arc
- Cache with RwLock
- State injection in commands
```

**Examples from Hestia**:
- Reference: `src-tauri/src/config/app.rs`
- Pattern: AppState struct, Arc<DatabaseManager>

#### Task 2.3: Create testing-guide.md
**Effort**: M (3-4 hours)
**Priority**: P1

**Objectives**:
- Document Cargo test framework
- Show #[tokio::test] for async tests
- Explain test module organization
- Document mocking strategies
- Show assertion macros
- Result-based test functions

**Acceptance Criteria**:
- [ ] Covers Cargo test basics
- [ ] Documents async testing
- [ ] Shows mocking patterns
- [ ] Includes test organization
- [ ] Documents assertions
- [ ] At least 8 complete examples

**Content Structure**:
```markdown
# Testing Rust Backend Code

## Test Organization
- #[cfg(test)] modules
- Test file placement
- Integration vs unit tests

## Writing Tests
- #[test] attribute
- #[tokio::test] for async
- Test function signatures
- Result<()> return types

## Assertions
- assert! macro
- assert_eq! macro
- assert_ne! macro
- matches! macro
- Custom assertions

## Mocking
- Trait-based mocking
- mockall crate
- Test doubles
- Dependency injection

## Test Utilities
- tempfile and tempdir
- Test fixtures
- Setup and teardown

## Testing Async Code
- tokio::test macro
- Testing timeouts
- Testing concurrent operations

## Complete Examples
- Unit test examples
- Integration test examples
- Async test examples
- Mock usage examples
- Tests from Hestia watcher
```

**Examples from Hestia**:
- Reference: `src-tauri/src/tests/watcher.rs`
- Patterns: tokio::test, tempdir, Result-based tests

---

### Phase 3: Advanced Topics (Files 7-9) - MEDIUM PRIORITY
**Estimated Effort**: L (12-14 hours)
**Dependencies**: Phase 2 complete
**Risk Level**: Low

#### Task 3.1: Create tracing-logging.md
**Effort**: M (3-4 hours)
**Priority**: P2

**Objectives**:
- Document tracing macros (info!, debug!, warn!, error!)
- Show #[instrument] macro usage
- Explain structured logging with fields
- Document tracing-subscriber setup
- Show context propagation
- Performance tracking with spans

**Acceptance Criteria**:
- [ ] Covers tracing basics
- [ ] Documents #[instrument] usage
- [ ] Shows structured logging
- [ ] Includes subscriber setup
- [ ] Documents span management
- [ ] At least 5 complete examples

**Content Structure**:
```markdown
# Tracing and Logging

## Tracing Overview
- tracing vs log crate
- Structured logging benefits
- Span and event model

## Tracing Macros
- info!, debug!, warn!, error!
- Structured fields
- Dynamic fields
- Log levels

## Instrumentation
- #[instrument] macro
- Skip parameters
- Custom span names
- Return value logging

## Subscriber Setup
- tracing-subscriber
- Formatting subscribers
- Filter configuration
- Multiple subscribers

## Spans
- Creating spans
- Entering spans
- Span nesting
- Async span propagation

## Complete Examples
- Basic logging
- Instrumented functions
- Structured fields
- Performance tracking
```

**Examples**:
- Based on Rust tracing documentation
- Pattern: Instrumenting database operations

#### Task 3.2: Create type-driven-design.md
**Effort**: L (4-5 hours)
**Priority**: P2

**Objectives**:
- Document newtype pattern
- Show enum usage for ADTs
- Explain Option/Result transforms
- Document From/Into traits
- Show AsRef pattern
- Making illegal states unrepresentable

**Acceptance Criteria**:
- [ ] Covers newtype pattern
- [ ] Documents ADT patterns
- [ ] Shows Option/Result usage
- [ ] Includes trait implementations
- [ ] Documents type safety principles
- [ ] At least 7 complete examples

**Content Structure**:
```markdown
# Type-Driven Design in Rust

## Type System Philosophy
- Making illegal states unrepresentable
- Encoding constraints at compile time
- Type-driven development

## Newtype Pattern
- Wrapping primitive types
- Semantic meaning
- Validation in constructors
- Transparent wrapper

## Algebraic Data Types
- Enum variants with data
- Pattern matching
- Exhaustive checks

## Option and Result
- map, and_then, unwrap_or
- Chaining operations
- Avoiding explicit match

## Trait-Based Design
- From/Into implementations
- AsRef for conversions
- Custom traits

## Complete Examples
- Email newtype
- Status enum
- Error handling with Result
- Conversion traits
- Examples from Hestia
```

**Examples from Hestia**:
- Reference: `src-tauri/src/data/`, `src-tauri/src/file_system/`
- Patterns: FileEvent, EventKind enums

#### Task 3.3: Create ownership-patterns.md
**Effort**: M (3-4 hours)
**Priority**: P2

**Objectives**:
- Document borrowing rules (& and &mut)
- Explain lifetimes in functions
- Show Arc for shared ownership
- Document Clone strategies
- Explain moving vs borrowing
- Common ownership patterns in async code

**Acceptance Criteria**:
- [ ] Covers borrowing basics
- [ ] Documents lifetime annotations
- [ ] Shows Arc patterns
- [ ] Explains Clone usage
- [ ] Documents async ownership
- [ ] At least 6 complete examples

**Content Structure**:
```markdown
# Ownership and Borrowing Patterns

## Ownership Rules
- Move semantics
- Ownership transfer
- Drop trait

## Borrowing
- Immutable references (&)
- Mutable references (&mut)
- Borrowing rules
- Common errors

## Lifetimes
- Lifetime annotations
- Function signatures
- Struct lifetimes
- 'static lifetime

## Shared Ownership
- Arc for thread-safe sharing
- Rc for single-threaded
- When to use each
- Performance considerations

## Clone Strategies
- When to derive Clone
- Cheap vs expensive clones
- Cow (Clone on Write)

## Async Ownership
- Send + 'static bounds
- Sharing data across tasks
- Avoiding lifetime issues

## Complete Examples
- Function borrowing
- Arc usage patterns
- Lifetime annotations
- Async task ownership
```

**Examples from Hestia**:
- Pattern: Arc<DatabaseManager>, FileOperations ownership

---

### Phase 4: Integration & Examples (File 10) - FINAL POLISH
**Estimated Effort**: M (6-8 hours)
**Dependencies**: Phases 1-3 complete
**Risk Level**: Low

#### Task 4.1: Create complete-examples.md
**Effort**: M (6-8 hours)
**Priority**: P3

**Objectives**:
- Provide end-to-end feature examples
- Show complete CRUD operations
- Document full error handling flow
- Include testing examples
- Base all examples on actual Hestia code

**Acceptance Criteria**:
- [ ] At least 3 complete feature examples
- [ ] Full command-to-database flows
- [ ] Error handling throughout
- [ ] Testing for each example
- [ ] All code compiles
- [ ] Draws from actual Hestia code

**Content Structure**:
```markdown
# Complete Examples

## Example 1: Tag Management Feature
- Command handler
- Service layer
- Database operations
- Error handling
- Testing

## Example 2: File Scanning Feature
- Async command
- Background processing
- State management
- Error handling
- Testing

## Example 3: CRUD Operations
- Create operation
- Read operation
- Update operation
- Delete operation
- Full testing suite

## Example 4: Complex Query
- Multiple joins
- Filtering
- Pagination
- Error handling

## Refactoring Guide
- From bad patterns to good
- Step-by-step improvements
```

**Examples from Hestia**:
- Tag creation flow: commands → service → database
- File scanning: async patterns, state management
- Actual working code from Hestia

---

### Phase 5: Quality Assurance & Validation
**Estimated Effort**: M (4-6 hours)
**Dependencies**: All phases complete
**Risk Level**: Low

#### Task 5.1: Cross-Reference Validation
**Effort**: S (1-2 hours)
**Priority**: P1

**Objectives**:
- Verify all links between files work
- Ensure consistency across all resources
- Check that SKILL.md references are correct

**Acceptance Criteria**:
- [ ] All navigation links work
- [ ] No broken references
- [ ] Consistent terminology
- [ ] Progressive disclosure maintained

#### Task 5.2: Code Compilation Check
**Effort**: M (2-3 hours)
**Priority**: P0 - Blocking

**Objectives**:
- Extract all code examples
- Verify they compile
- Run any testable examples

**Acceptance Criteria**:
- [ ] All code examples are syntactically correct
- [ ] Examples compile with appropriate dependencies
- [ ] Test examples run successfully

#### Task 5.3: CLAUDE.md Alignment Check
**Effort**: S (1 hour)
**Priority**: P1

**Objectives**:
- Verify all CLAUDE.md guidelines are covered
- Ensure type-driven development principles included
- Check error handling best practices

**Acceptance Criteria**:
- [ ] All design guidelines represented
- [ ] Implementation rules covered
- [ ] TDD practices documented

---

## Risk Assessment and Mitigation

### Technical Risks

#### Risk 1: Code Examples Don't Compile
**Probability**: Medium | **Impact**: High | **Overall**: High

**Mitigation**:
- Test all code examples in a separate Rust project
- Use actual Hestia code as source material
- Have compilation check phase (Task 5.2)

**Contingency**:
- Create a test project with all dependencies
- Compile-test each example before committing

#### Risk 2: Inconsistency Across Resource Files
**Probability**: Medium | **Impact**: Medium | **Overall**: Medium

**Mitigation**:
- Use consistent terminology throughout
- Cross-reference validation phase (Task 5.1)
- Single source of truth for patterns

**Contingency**:
- Create a glossary of terms
- Regular review of completed files

#### Risk 3: Missing Hestia Patterns
**Probability**: Low | **Impact**: Medium | **Overall**: Low

**Mitigation**:
- Comprehensive Hestia codebase analysis already done
- Reference actual file paths in examples
- Validate against running Hestia code

**Contingency**:
- Review Hestia codebase again for missed patterns
- Iterate on examples based on user feedback

### Process Risks

#### Risk 4: Scope Creep
**Probability**: Medium | **Impact**: Medium | **Overall**: Medium

**Mitigation**:
- Clear task definitions with acceptance criteria
- Phase-based approach
- Focus on Hestia patterns only

**Contingency**:
- Defer non-essential examples to future iterations
- Create "Future Enhancements" section

#### Risk 5: Time Estimation Inaccuracy
**Probability**: High | **Impact**: Low | **Overall**: Medium

**Mitigation**:
- Buffer time in estimates
- Track actual time vs estimated
- Adjust subsequent phase estimates

**Contingency**:
- Prioritize critical path (Phase 1)
- Defer lower priority files if needed

---

## Success Metrics

### Quantitative Metrics

1. **Code Quality**
   - Target: 100% of code examples compile
   - Measurement: Compilation check phase
   - Baseline: 0% (currently TypeScript)

2. **Coverage**
   - Target: 100% of Hestia patterns documented
   - Measurement: Pattern checklist
   - Baseline: Comprehensive analysis already done

3. **Completeness**
   - Target: 0 TypeScript references remaining
   - Measurement: Text search for TypeScript keywords
   - Baseline: 290 lines of TypeScript patterns

4. **Alignment**
   - Target: 100% alignment with CLAUDE.md guidelines
   - Measurement: Guideline checklist
   - Baseline: Design guidelines present, implementation varies

### Qualitative Metrics

1. **Usability**
   - Copy-paste ready examples
   - Clear explanations
   - Progressive disclosure maintained

2. **Maintainability**
   - Consistent structure across files
   - Clear organization
   - Easy to update

3. **Educational Value**
   - Explains "why" not just "how"
   - Based on real working code
   - Covers common pitfalls

---

## Required Resources and Dependencies

### Human Resources

**Primary Developer** (You):
- Rust expertise
- Tauri knowledge
- Technical writing skills
- Estimated: 50-60 hours total

**Reviewer** (User):
- Domain expertise in Hestia
- Validation of patterns
- Feedback on clarity

### Technical Resources

**Reference Materials**:
- ✅ Hestia codebase (already analyzed)
- ✅ Rust book documentation (fetched from Context7)
- ✅ Tokio documentation (fetched from Context7)
- ✅ SeaORM documentation (fetched from Context7)
- ✅ Tauri documentation (fetched from Context7)

**Development Tools**:
- Rust compiler for testing examples
- Claude Code for development
- Git for version control

### Dependencies

**External Dependencies**:
- None (all self-contained)

**Internal Dependencies**:
- Phase 1 must complete before Phase 2
- Phase 2 must complete before Phase 3
- All phases must complete before Phase 4
- Phase 5 runs after all content phases

---

## Timeline Estimates

### Phase-Based Timeline

| Phase | Tasks | Effort | Duration | Dependencies |
|-------|-------|--------|----------|--------------|
| Phase 1 | 4 tasks | XL | 20-24 hours | None |
| Phase 2 | 3 tasks | L | 12-14 hours | Phase 1 |
| Phase 3 | 3 tasks | L | 12-14 hours | Phase 2 |
| Phase 4 | 1 task | M | 6-8 hours | Phases 1-3 |
| Phase 5 | 3 tasks | M | 4-6 hours | Phase 4 |
| **Total** | **14 tasks** | **XL** | **54-66 hours** | |

### Critical Path

The critical path through the project:
1. Task 1.1: Update SKILL.md (4-5 hours) - BLOCKING
2. Task 1.2: Create tauri-commands.md (4-5 hours) - BLOCKING
3. Task 1.3: Create error-handling.md (4-5 hours) - BLOCKING
4. Task 1.4: Create seaorm-database.md (6-8 hours) - BLOCKING
5. Task 5.2: Code Compilation Check (2-3 hours) - BLOCKING

**Critical Path Total**: 20-26 hours

All other tasks can be parallelized or done in any order after their phase dependencies are met.

### Recommended Schedule

**Week 1** (Phase 1):
- Days 1-2: Tasks 1.1, 1.2
- Days 3-4: Task 1.3
- Days 4-5: Task 1.4

**Week 2** (Phase 2):
- Days 1-2: Tasks 2.1, 2.2
- Day 3: Task 2.3

**Week 3** (Phases 3-5):
- Days 1-2: Tasks 3.1, 3.2, 3.3
- Days 3-4: Task 4.1
- Day 5: Phase 5 (Tasks 5.1, 5.2, 5.3)

---

## Appendix A: File-by-File Transformation Map

| Old File | New File | Status | Notes |
|----------|----------|--------|-------|
| SKILL.md | SKILL.md | Transform | Update all sections |
| architecture-overview.md | (merge into SKILL.md) | Merge | Core architecture in main file |
| routing-and-controllers.md | tauri-commands.md | Replace | Completely different paradigm |
| async-and-errors.md | error-handling.md | Split | Errors separate from async |
| (new) | async-patterns.md | Create | New file for tokio patterns |
| testing-guide.md | testing-guide.md | Replace | Complete rewrite for Cargo |
| database-patterns.md | seaorm-database.md | Replace | Prisma → SeaORM |
| configuration.md | (merge into state-management.md) | Merge | Config is part of state |
| validation-patterns.md | (merge into type-driven-design.md) | Merge | Validation through types |
| middleware-guide.md | (remove) | Delete | No direct equivalent in Tauri |
| sentry-and-monitoring.md | tracing-logging.md | Replace | Sentry → tracing |
| services-and-repositories.md | (merge into seaorm-database.md) | Merge | Simplified architecture |
| (new) | state-management.md | Create | Arc/Mutex/AppState patterns |
| (new) | type-driven-design.md | Create | Rust type system emphasis |
| (new) | ownership-patterns.md | Create | Rust-specific ownership |
| complete-examples.md | complete-examples.md | Replace | All new Rust examples |

---

## Appendix B: Pattern Mapping Reference

### TypeScript → Rust Pattern Mappings

| TypeScript Pattern | Rust Equivalent | File Location |
|-------------------|-----------------|---------------|
| `export class XController extends BaseController` | `#[command] pub async fn` | tauri-commands.md |
| `try { ... } catch (e) { ... }` | `Result<T, E>` + `?` | error-handling.md |
| `async/await with Promise` | `async/await with tokio` | async-patterns.md |
| `PrismaClient.user.findMany()` | `Entity::find().all(&db)` | seaorm-database.md |
| `describe('Test', () => { it(...) })` | `#[tokio::test] async fn` | testing-guide.md |
| `z.object({ email: z.string() })` | Manual validation or newtype | type-driven-design.md |
| `process.env.VAR` | `config.var` or `std::env::var` | state-management.md |
| `import * as Sentry` | `use tracing::{info, error}` | tracing-logging.md |
| `express.Router()` | `tauri::generate_handler!` | tauri-commands.md |
| `req: Request, res: Response` | `State<'_, Mutex<AppState>>` | tauri-commands.md |

---

## Appendix C: Hestia Pattern Reference

### Key Files to Reference

**Command Handlers**:
- `src-tauri/src/commands/tag_management.rs`
- `src-tauri/src/commands/watched_folder_management.rs`

**Database Operations**:
- `src-tauri/src/database/operations.rs`
- `src-tauri/src/database/manager.rs`

**Error Types**:
- `src-tauri/src/errors/db.rs`
- `src-tauri/src/errors/app.rs`

**State Management**:
- `src-tauri/src/config/app.rs`

**Testing**:
- `src-tauri/src/tests/watcher.rs`

**Async Patterns**:
- `src-tauri/src/file_system/watcher.rs`

---

**End of Plan Document**

This plan provides a comprehensive roadmap for transforming the backend-dev-guidelines skill from TypeScript to Rust. Each phase builds on the previous one, with clear acceptance criteria and examples drawn from the actual Hestia codebase.
