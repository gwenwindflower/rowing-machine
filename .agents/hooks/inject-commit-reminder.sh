#!/usr/bin/env bash
#
# SubagentStart hook: inject commit instructions when a subagent spawns.
#
# Fires on every SubagentStart event. Only acts when teammate_name is set
# (subagent on a team). Injects additionalContext reminding the agent to
# commit their work before marking the task complete.

set -euo pipefail

INPUT=$(cat)

# Only apply to teammates (subagents on a team), not standalone agents
teammate_name=$(echo "$INPUT" | jq -r '.teammate_name // empty')
if [ -z "$teammate_name" ]; then
  exit 0
fi

# Inject commit reminder as additional context
cat <<'EOF'
{
  "hookSpecificOutput": {
    "additionalContext": "IMPORTANT: You are working in an isolated git worktree. Before marking any task as complete, you MUST commit your changes:\n\n  git add -A\n  git commit -m \"feat(scope): describe what you built\"\n\nUse a descriptive commit message. A TaskCompleted hook will BLOCK you from completing if you have uncommitted changes."
  }
}
EOF
