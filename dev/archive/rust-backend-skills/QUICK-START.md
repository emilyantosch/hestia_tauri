# Quick Start: Resume Rust Backend Skills Transformation

**Status**: Phase 1 - 50% Complete
**Next**: Task 1.3 - Create error-handling.md
**Updated**: 2025-11-02 20:53:48

---

## ⚡ Start Here

### Next File to Create
`.claude/skills/backend-dev-guidelines/resources/error-handling.md`

### Read These Files First
1. `src-tauri/src/errors/db.rs` - DbError patterns
2. `src-tauri/src/errors/app.rs` - AppError patterns

### Structure Template
See `rust-backend-skills-tasks.md` lines 97-114

---

## 🎯 Key Requirements

### Must Include
- Error philosophy (anyhow vs thiserror)
- Error enum variants (unit, tuple, struct)
- #[from] conversions
- anyhow::Context usage
- Serialize implementation
- ? operator patterns
- 6+ complete examples

### Critical Pattern
- **anyhow::Result<T>** = Internal operations
- **Result<T, E>** = IPC endpoints only

---

## ⚠️ User's Updates (MUST OBSERVE)

### Architecture
Command → **Controller** → Service → Repository → Database

### 8 Principles
1. Commands/Controllers delegate, Services implement
2. **anyhow::Result for internal, Result<T, E> for IPC**
3. Type system for illegal states
4. **All types in app/data**
5. Arc for sharing
6. thiserror for domain errors
7. Instrument with tracing
8. **std::test preferred, property-based testing**

### Directory
```
controllers/     # NEW
database/
  repository.rs  # Was operations.rs
data/
  internal/      # Backend types
  commands/      # IPC types
```

---

## 📊 Progress

- Phase 1: 2/4 (50%)
- Total: 2/14 (14%)
- Completed: SKILL.md, tauri-commands.md
- Next: error-handling.md
- Then: seaorm-database.md

---

## 📚 Documentation

- Full Details: `HANDOFF-2025-11-02.md`
- Session Summary: `SESSION-SUMMARY-2025-11-02-part2.md`
- Context: `rust-backend-skills-context.md`
- Tasks: `rust-backend-skills-tasks.md`

---

**Ready to Code**: Create error-handling.md now!
