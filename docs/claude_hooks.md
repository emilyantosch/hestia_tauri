# Claude Code Hooks System

Claude Code hooks are automated scripts that run at specific lifecycle events during your coding session. Hooks are registered in `.claude/settings.json` and execute shell wrappers (`.sh`) which pipe JSON input via stdin to TypeScript files (`.ts`) for complex processing. The TypeScript files analyze the input, perform logic like skill matching or error detection, and output formatted messages to stdout which Claude displays to you. This architecture separates fast bash pre-checks from complex TypeScript logic while maintaining a simple stdin/stdout interface.

The system uses a two-layer design: shell wrappers handle environment variables, path resolution, and runtime selection (Deno in this project), while TypeScript files handle the actual business logic like parsing skill rules, analyzing file changes, and matching intent patterns. Claude sends structured JSON data to the shell script, which pipes it to Deno running the TypeScript file, and any output from `console.log` is captured and displayed back to you.

## Flow Chart

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. User Action (submit prompt, edit file, or stop)             │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Claude Code checks .claude/settings.json for hooks          │
│    - UserPromptSubmit, PostToolUse, Stop events                │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. Execute Shell Wrapper (.sh)                                  │
│    ├─ Check environment variables (e.g., SKIP_ERROR_REMINDER)  │
│    ├─ Set working directory                                     │
│    └─ Pipe stdin: cat | deno run --allow-all script.ts         │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼ (JSON via stdin)
┌─────────────────────────────────────────────────────────────────┐
│ 4. TypeScript Processing (.ts)                                  │
│    ├─ Read stdin: readFileSync(0, 'utf-8')                     │
│    ├─ Parse JSON input                                          │
│    ├─ Execute logic (match skills, analyze files, check types) │
│    └─ Output to stdout: console.log(results)                    │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼ (Output via stdout)
┌─────────────────────────────────────────────────────────────────┐
│ 5. Claude Code captures output and displays to user            │
└─────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

| Component | Purpose | Examples |
|-----------|---------|----------|
| **settings.json** | Register hooks and map events to shell scripts | `UserPromptSubmit` → `skill-activation-prompt.sh` |
| **Shell Wrapper (.sh)** | Fast pre-checks, environment setup, runtime execution | Check `SKIP_ERROR_REMINDER`, run `deno` with permissions |
| **TypeScript (.ts)** | Complex logic, JSON parsing, file analysis | Parse `skill-rules.json`, regex matching, file content analysis |
| **stdin** | Input channel from Claude to TypeScript | JSON with `{session_id, prompt, cwd, ...}` |
| **stdout** | Output channel from TypeScript to Claude | Formatted messages, skill suggestions, error reports |
| **Deno Runtime** | Execute TypeScript without compilation | `deno run --allow-all` with native TS support |

## Configured Hooks

| Event | Hook Script | Purpose |
|-------|-------------|---------|
| **UserPromptSubmit** | `skill-activation-prompt.sh` | Analyzes your prompt and suggests relevant skills to activate |
| **PostToolUse** (Edit/Write) | `post-tool-use-tracker.sh` | Tracks file edits for later analysis |
| **Stop** | `tsc-check.sh` | Checks TypeScript compilation errors |
| **Stop** | `trigger-build-resolver.sh` | Suggests fixes if build errors are detected |
