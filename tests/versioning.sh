#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
sandbox="$(mktemp -d "${TMPDIR:-/tmp}/versioning.XXXXXX")"
trap 'rm -rf "$sandbox"' EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

assert_contains() {
	[[ "$1" == *"$2"* ]] || fail "expected output to contain: $2"
}

# A fake language kit: VERSION is the declared version, DERIVED is a file write must keep in sync.
make_repo() {
	local name="$1" version="$2" derived="$3"
	local root="$sandbox/$name"

	mkdir -p "$root/mise-tasks"
	cp -R "$repo_root/mise-tasks/version" "$root/mise-tasks/version"
	cp -R "$repo_root/mise-tasks/release" "$root/mise-tasks/release"
	printf '%s\n' "$version" >"$root/VERSION"
	printf '%s\n' "$derived" >"$root/DERIVED"

	cat >"$root/mise-tasks/version/read" <<'HOOK'
#!/usr/bin/env bash
cat VERSION
HOOK
	cat >"$root/mise-tasks/version/write" <<'HOOK'
#!/usr/bin/env bash
printf '%s\n' "$1" >VERSION
printf '%s\n' "$1" >DERIVED
HOOK
	cat >"$root/mise-tasks/version/files" <<'HOOK'
#!/usr/bin/env bash
printf 'VERSION\nDERIVED\n'
HOOK
	cat >"$root/mise-tasks/version/verify" <<'HOOK'
#!/usr/bin/env bash
[[ "$(cat VERSION)" == "$(cat DERIVED)" ]]
HOOK
	chmod +x "$root"/mise-tasks/version/{read,write,files,verify}
	printf '%s\n' "$root"
}

run_task() {
	local root="$1" task="$2"
	shift 2
	(
		cd "$root"
		export MISE_PROJECT_ROOT="$root"
		export GIT_AUTHOR_NAME=test GIT_AUTHOR_EMAIL=test@example.com
		export GIT_COMMITTER_NAME=test GIT_COMMITTER_EMAIL=test@example.com
		export GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=commit.gpgsign GIT_CONFIG_VALUE_0=false
		"$root/mise-tasks/$task" "$@"
	) 2>&1
}

synced="$(make_repo synced 1.2.3 1.2.3)"
run_task "$synced" version/check >/dev/null || fail 'synchronized versions reported drift'
run_task "$synced" version/check v1.2.3 >/dev/null || fail 'matching tag reported drift'

set +e
mismatch="$(run_task "$synced" version/check v9.9.9)"
mismatch_status=$?
set -e
[[ "$mismatch_status" -ne 0 ]] || fail 'a tag the project does not declare should fail'
assert_contains "$mismatch" 'declares 1.2.3; v9.9.9 expects 9.9.9'

drifted="$(make_repo drifted 1.2.3 1.2.1)"
set +e
drift="$(run_task "$drifted" version/check)"
drift_status=$?
set -e
[[ "$drift_status" -ne 0 ]] || fail 'version drift should fail'
assert_contains "$drift" 'out of sync'

run_task "$drifted" version/sync >/dev/null || fail 'version/sync left the repository out of sync'
[[ "$(cat "$drifted/DERIVED")" == 1.2.3 ]] || fail 'version/sync did not repair the derived file'
[[ "$(cat "$drifted/VERSION")" == 1.2.3 ]] || fail 'version/sync changed the source of truth'

bump="$(make_repo bump 1.2.3 1.2.3)"
run_task "$bump" version/bump v1.3.0 >/dev/null || fail 'version/bump failed'
[[ "$(cat "$bump/VERSION")" == 1.3.0 ]] || fail 'version/bump did not set the declared version'
[[ "$(cat "$bump/DERIVED")" == 1.3.0 ]] || fail 'version/bump did not sync the derived file'
run_task "$bump" version/check v1.3.0 >/dev/null || fail 'version/bump left the repository unreleasable'

commit="$(make_repo commit 1.2.3 1.2.3)"
(
	cd "$commit"
	git init --quiet --initial-branch=main
	git -c user.name=test -c user.email=test@example.com -c commit.gpgsign=false add -A
	git -c user.name=test -c user.email=test@example.com -c commit.gpgsign=false commit --quiet -m 'feat: initial'
) || fail 'could not create the commit sandbox'
run_task "$commit" version/bump v1.3.0 >/dev/null
printf 'stray\n' >"$commit/STRAY"
set +e
stray="$(run_task "$commit" release/commit v1.3.0)"
stray_status=$?
set -e
[[ "$stray_status" -ne 0 ]] || fail 'release/commit accepted a change outside the version files'
assert_contains "$stray" 'Only version files may change'
rm "$commit/STRAY"
run_task "$commit" release/commit v1.3.0 >/dev/null || fail 'release/commit refused the version files'
[[ "$(cd "$commit" && git log -1 --format=%s)" == 'chore(release): prepare v1.3.0' ]] || fail 'release/commit wrote the wrong subject'
[[ -z "$(cd "$commit" && git status --porcelain)" ]] || fail 'release/commit left the tree dirty'

printf 'Versioning tests passed.\n'
