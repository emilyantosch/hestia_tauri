# Architecture Overview - Backend Services

Complete guide to the layered architecture pattern used in backend microservices.

## Table of Contents

- [Layered Architecture Pattern](#layered-architecture-pattern)
- [Request Lifecycle](#request-lifecycle)
- [Service Comparison](#service-comparison)
- [Directory Structure Rationale](#directory-structure-rationale)
- [Module Organization](#module-organization)
- [Separation of Concerns](#separation-of-concerns)

---

## Layered Architecture Pattern

### The Four Layers

```
┌─────────────────────────────────────┐
│         HTTP Request                │
└───────────────┬─────────────────────┘
                ↓
┌─────────────────────────────────────┐
│  Layer 1: ROUTES                    │
│  - Route definitions only           │
│  - Middleware registration          │
│  - Delegate to controllers          │
│  - NO business logic                │
└───────────────┬─────────────────────┘
                ↓
┌─────────────────────────────────────┐
│  Layer 2: CONTROLLERS               │
│  - Request/response handling        │
│  - Input validation                 │
│  - Call services                    │
│  - Format responses                 │
│  - Error handling                   │
└───────────────┬─────────────────────┘
                ↓
┌─────────────────────────────────────┐
│  Layer 3: SERVICES                  │
│  - Business logic                   │
│  - Orchestration                    │
│  - Call repositories                │
│  - No HTTP knowledge                │
└───────────────┬─────────────────────┘
                ↓
┌─────────────────────────────────────┐
│  Layer 4: REPOSITORIES              │
│  - Data access abstraction          │
│  - SQLx/SeaORM operations           │
│  - Query optimization               │
│  - Caching                          │
└───────────────┬─────────────────────┘
                ↓
┌─────────────────────────────────────┐
│    Database (SQLite/Postgres)       │
└─────────────────────────────────────┘
```

### Why This Architecture?

**Testability:**
- Each layer can be tested independently
- Easy to mock dependencies
- Clear test boundaries

**Maintainability:**
- Changes isolated to specific layers
- Business logic separate from HTTP concerns
- Easy to locate bugs

**Reusability:**
- Services can be used by routes, cron jobs, scripts
- Repositories hide database implementation
- Business logic not tied to HTTP

**Scalability:**
- Easy to add new endpoints
- Clear patterns to follow
- Consistent structure

---

## Request Lifecycle

### Complete Flow Example

```rust
1. HTTP POST /api/users
   ↓
2. Express matches route in userRoutes.ts
   ↓
3. Middleware chain executes:
   - SSOMiddleware.verifyLoginStatus (authentication)
   - auditMiddleware (context tracking)
   ↓
4. Route handler delegates to controller
   ↓
5. Controller validates and calls service:
   - Validate input
   - Call userService.create(data)
   - Handle success/error
   ↓
6. Service executes business logic:
   - Check business rules
   - Call userRepository.create(data)
   - Return result
   ↓
7. Repository performs database operation:
   - Handle database errors
   - Return created user
   ↓
8. Response flows back:
   Repository → Service → Controller → API → Client
```

### Middleware Execution Order

**Critical:** Middleware executes in registration order

```rust
// 1. Body parsing
// 2. URL encoding
// 3. Cookie parsing
// 4. Auth initialization
// ... routes registered here
// 6. Audit (if global)
```

**Rule:** Error handlers must be registered AFTER routes!

---

## Service Comparison

### Email Service (Mature Pattern ✅)

**Strengths:**
- Comprehensive BaseController
- Clean route delegation (no business logic in routes)
- Consistent dependency injection pattern using traits where it makes sense
- Good middleware organization
- Type-safe throughout
- Excellent error handling

**Example Structure:**
```
email/src/
├── controllers/
│   ├── BaseController.rs          ✅ Excellent template
│   ├── NotificationController.rs  ✅ Extends BaseController
│   └── EmailController.rs         ✅ Clean patterns
├── routes/
│   ├── notificationRoutes.rs      ✅ Clean delegation
│   └── emailRoutes.rs             ✅ No business logic
├── services/
│   ├── NotificationService.rs     ✅ Dependency injection
│   └── BatchingService.rs         ✅ Clear responsibility
└── middleware/
    ├── errorBoundary.rs           ✅ Comprehensive
    └── DevImpersonationSSOMiddleware.rs
```

**Use as template** for new services!

### Form Service (Transitioning ⚠️)

**Strengths:**
- Excellent workflow architecture
- Innovative audit middleware
- Comprehensive permission system

**Weaknesses:**
- Some routes have 200+ lines of business logic
- Inconsistent controller naming
- Direct process.env usage (60+ occurrences)
- Minimal repository pattern usage

**Example:**
```
form/src/
├── routes/
│   ├── responseRoutes.ts          ❌ Business logic in routes
│   └── proxyRoutes.ts             ✅ Good validation pattern
├── controllers/
│   ├── formController.ts          ⚠️ Lowercase naming
│   └── UserProfileController.ts   ✅ PascalCase naming
├── workflow/                      ✅ Excellent architecture!
│   ├── core/
│   │   ├── WorkflowEngineV3.ts   ✅ Event sourcing
│   │   └── DryRunWrapper.ts      ✅ Innovative
│   └── services/
└── middleware/
    └── auditMiddleware.ts         ✅ AsyncLocalStorage pattern
```

**Learn from:** workflow/, middleware/auditMiddleware.rs
**Avoid:** responseRoutes.rs, direct process.env

---

## Directory Structure Rationale

### Controllers Directory

**Purpose:** Handle HTTP request/response concerns

**Contents:**
- `BaseController.rs` - Base class with common methods
- `{Feature}Controller.rs` - Feature-specific controllers

**Naming:** PascalCase + Controller

**Responsibilities:**
- Parse request parameters
- Validate input
- Call appropriate service methods
- Format responses
- Handle errors (via BaseController)
- Set HTTP status codes

### Services Directory

**Purpose:** Business logic and orchestration

**Contents:**
- `{feature}Service.rs` - Feature business logic

**Naming:** camelCase + Service (or PascalCase + Service)

**Responsibilities:**
- Implement business rules
- Orchestrate multiple repositories
- Transaction management
- Business validations
- No HTTP knowledge (Request/Response types)

### Repositories Directory

**Purpose:** Data access abstraction

**Contents:**
- `{Entity}Repository.rs` - Database operations for entity

**Naming:** PascalCase + Repository

**Responsibilities:**
- Prisma query operations
- Query optimization
- Database error handling
- Caching layer
- Hide Prisma implementation details

**Current Gap:** Only 1 repository exists (WorkflowRepository)

### Routes Directory

**Purpose:** Route registration ONLY

**Contents:**
- `{feature}Routes.rs` - Express router for feature

**Naming:** camelCase + Routes

**Responsibilities:**
- Register routes with Express
- Apply middleware
- Delegate to controllers
- **NO business logic!**

### Middleware Directory

**Purpose:** Cross-cutting concerns

**Contents:**
- Authentication middleware
- Audit middleware
- Error boundaries
- Validation middleware
- Custom middleware

**Naming:** camelCase

**Types:**
- Request processing (before handler)
- Response processing (after handler)
- Error handling (error boundary)

### Config Directory

**Purpose:** Configuration management

**Contents:**
- `unifiedConfig.rs` - Type-safe configuration
- Environment-specific configs

**Pattern:** Single source of truth

### Types Directory

**Purpose:** TypeScript type definitions

**Contents:**
- `{feature}.types.rs` - Feature-specific types
- DTOs (Data Transfer Objects)
- Request/Response types
- Domain models

---

## Module Organization

### Feature-Based Organization

For large features, use subdirectories:

```
src/workflow/
├── core/              # Core engine
├── services/          # Workflow-specific services
├── actions/           # System actions
├── models/            # Domain models
├── validators/        # Workflow validation
└── utils/             # Workflow utilities
```

**When to use:**
- Feature has 5+ files
- Clear sub-domains exist
- Logical grouping improves clarity

### Flat Organization

For simple features:

```
src/
├── controllers/UserController.rs
├── services/userService.rs
├── routes/userRoutes.rs
└── repositories/UserRepository.rs
```

**When to use:**
- Simple features (< 5 files)
- No clear sub-domains
- Flat structure is clearer

---

## Separation of Concerns

### What Goes Where

**Routes Layer:**
- ✅ Route definitions
- ✅ Middleware registration
- ✅ Controller delegation
- ❌ Business logic
- ❌ Database operations
- ❌ Validation logic (should be in validator or controller)

**Controllers Layer:**
- ✅ Request parsing (params, body, query)
- ✅ Input validation
- ✅ Service calls
- ✅ Response formatting
- ✅ Error handling
- ❌ Business logic
- ❌ Database operations

**Services Layer:**
- ✅ Business logic
- ✅ Business rules enforcement
- ✅ Orchestration (multiple repos)
- ✅ Transaction management
- ❌ HTTP concerns (Request/Response)

- ❌ Direct Prisma calls (use repositories)
**Repositories Layer:**
- ✅ SQLx/SeaORM operations
- ✅ Query construction
- ✅ Database error handling
- ✅ Caching
- ❌ Business logic
- ❌ HTTP concerns

### Example: User Creation

**Route:**
```rust
Add example for route definition here
```

**Controller:**
```rust
Add example for user controller
```

**Service:**
```rust
Add example for user creation service
```

**Repository:**
```rust
Add repository end point example that handles DB interaction
```

**Notice:** Each layer has clear, distinct responsibilities!

---

**Related Files:**
- [SKILL.md](SKILL.md) - Main guide
- [routing-and-controllers.md](routing-and-controllers.md) - Routes and controllers details
- [services-and-repositories.md](services-and-repositories.md) - Service and repository patterns
