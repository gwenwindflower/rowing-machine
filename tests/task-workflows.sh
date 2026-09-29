#!/usr/bin/env bash

set -euo pipefail
cd "${MISE_PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

ci_tasks=()
while IFS= read -r invocation; do
	invocation="${invocation//\'/}"
	invocation="${invocation//\"/}"
	[[ "$invocation" != *' '* ]] || fail "CI task calls must select one named task: $invocation"
	ci_tasks+=("$invocation")
done < <(awk '/run: mise run / { sub(/^.*run: mise run /, ""); print }' .github/workflows/ci.yml)
[[ ${#ci_tasks[@]} -gt 0 ]] || fail "CI does not select any mise tasks"

ci_output=""
for task in "${ci_tasks[@]}"; do
	ci_output+="$(mise run --dry-run "$task" 2>&1)"$'\n'
done
for task in check test 'test:*' release:check CI; do
	if [[ "$task" == CI ]]; then
		output="$ci_output"
	else
		output="$(mise run --dry-run "$task" 2>&1)"
	fi
	if [[ "$output" == *'[dev:'* || "$output" == *'cargo pretty'* || "$output" == *'cargo install'* || "$output" == *'cargo binstall'* || "$output" == *'cargo publish'* ]]; then
		fail "$task selects an interactive, installation, or publication task"
	fi
	if [[ "$task" == check || "$task" == release:check || "$task" == CI ]]; then
		count="$(grep -Fc 'prek run --all-files' <<<"$output" || true)"
		[[ "$count" == 1 ]] || fail "$task must run the hook sweep exactly once (found ${count:-0})"
	fi
	for command in 'cargo test --all-features --locked' 'cargo build --locked --release' 'cargo package --locked --allow-dirty' './tests/task-workflows.sh'; do
		count="$(grep -Fc "$command" <<<"$output" || true)"
		[[ "$count" == 1 ]] || fail "$task must run $command exactly once (found ${count:-0})"
	done
done

printf 'Automated task selection tests passed.\n'
