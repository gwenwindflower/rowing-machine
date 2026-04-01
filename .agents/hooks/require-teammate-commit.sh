#!/usr/bin/env bash
#
# TaskCompleted hook: block teammates from completing tasks with uncommitted work.
#
# Fires on every TaskCompleted event. Only acts when:
#   1. teammate_name is set (this is a subagent on a team, not the main thread)
#   2. We're in a git worktree
#   3. The worktree has uncommitted changes
#
# If all three are true, blocks the completion with exit 2 and tells the agent
# to commit before marking the task done.

set -euo pipefail

INPUT=$(cat)

# Only apply to teammates (subagents), not the main thread
teammate_name=$(echo "$INPUT" | jq -r '.teammate_name // empty')
if [ -z "$teammate_name" ]; then
  exit 0
fi

# Only apply inside worktrees
git_dir=$(git rev-parse --git-dir 2>/dev/null) || exit 0
if [[ "$git_dir" != *".git/worktrees/"* ]]; then
  exit 0
fi

# Check for uncommitted changes (staged, unstaged, or untracked)
if git diff --quiet && git diff --cached --quiet && [ -z "$(git ls-files --others --exclude-standard)" ]; then
  # Clean tree — all good
  exit 0
fi

# Dirty worktree — block completion
cat >&2 <<'MSG'
You have uncommitted changes in your worktree. Commit your work before marking this task complete:

  git add -A
  git commit -m "feat(scope): describe what you built"

Use a meaningful commit message that describes the changes, not a generic placeholder.
MSG
exit 2
