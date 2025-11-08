#!/bin/bash
set -e

cd "$CLAUDE_PROJECT_DIR/.claude/hooks"
cat | deno run --allow-all skill-activation-prompt.ts
