# Development Task Management

This directory contains active development tasks and planning documentation for the Hestia project.

## Directory Structure

```
dev/
├── README.md                    # This file
└── active/                      # Active tasks
    └── [task-name]/            # Individual task directory
        ├── [task-name]-plan.md     # Comprehensive plan
        ├── [task-name]-context.md  # Context and decisions
        └── [task-name]-tasks.md    # Task checklist
```

## Task Structure

Each task follows a three-document structure designed to survive context resets and provide complete information:

### 1. Plan Document (`[task-name]-plan.md`)
**Purpose**: Comprehensive strategic plan

**Contains**:
- Executive Summary
- Current State Analysis
- Proposed Future State
- Implementation Phases (detailed breakdown)
- Risk Assessment
- Success Metrics
- Timeline Estimates
- Appendices with reference materials

**Use when**: You need the full picture of what needs to be done and why

### 2. Context Document (`[task-name]-context.md`)
**Purpose**: Key decisions, files, and background information

**Contains**:
- Project context and objectives
- Key files and their locations
- Critical decisions made
- Dependencies and constraints
- Pattern references
- Common pitfalls to avoid
- Testing strategy
- Glossary of terms

**Use when**: You need to understand the reasoning behind decisions or find specific files

### 3. Tasks Document (`[task-name]-tasks.md`)
**Purpose**: Trackable checklist of all work items

**Contains**:
- Phase-organized task list
- Checkbox format for progress tracking
- Effort estimates
- Priority levels
- Dependencies
- Acceptance criteria
- Progress summary
- Issues log

**Use when**: You need to track progress or see what's next

## Creating a New Task

Use the `/dev-docs` slash command to create a new task:

```
/dev-docs Implement feature X with requirements Y
```

This will automatically:
1. Create the task directory structure
2. Generate all three documents
3. Populate them with relevant information

## Working with Tasks

### Starting Work
1. Read the **plan document** first for full context
2. Review the **context document** for key decisions
3. Use the **tasks document** for step-by-step execution

### During Work
- Update task checkboxes in the tasks document
- Log any issues or blockers in the Issues Log
- Add notes about decisions or changes

### Context Resets
If context is lost:
1. Open the task directory
2. Read the context document for quick orientation
3. Check the tasks document for current progress
4. Reference the plan document for detailed information

## Task Status

### Active Tasks
Located in `dev/active/[task-name]/`

**Current Active Tasks**:
- `rust-backend-skills/` - Transform backend skills from TypeScript to Rust
  - Status: Phase 1 100% COMPLETE ✅ (4/4 tasks, 100%)
  - Last Updated: 2025-11-04 15:30:00
  - Completed: Task 1.1, 1.2, 1.3, 1.4 (All Phase 1 resources created)
  - Next: Phase 2 - async-patterns.md, state-management.md, testing-guide.md

### Completed Tasks
Move to `dev/completed/[task-name]/` when done

### Archived Tasks
Move to `dev/archive/[task-name]/` if cancelled or superseded

## Best Practices

### Document Maintenance
- Keep "Last Updated" dates current
- Update progress regularly
- Log all significant decisions
- Cross-reference related tasks

### Task Breakdown
- Break large tasks into phases
- Assign effort estimates (S/M/L/XL)
- Set clear acceptance criteria
- Identify dependencies early

### Progress Tracking
- Use checkboxes consistently
- Update status after each work session
- Note blockers immediately
- Celebrate phase completions

## Integration with Claude Code

These documents are designed to work seamlessly with Claude Code:

**Skills**: Reference these documents in skill activation
**Hooks**: Can trigger based on file changes in task directories
**Commands**: `/dev-docs` creates new tasks automatically

## Questions?

Refer to:
- **Plan document**: For "what" and "why"
- **Context document**: For "how" and "where"
- **Tasks document**: For "when" and "status"

---

**Last Updated**: 2025-11-02
