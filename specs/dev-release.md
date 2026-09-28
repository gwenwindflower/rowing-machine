# Release and repository plumbing

## Goals

Every release is cut from a clean `main` by a human-gated pipeline that a contributor can rehearse without side effects. The version has one source of truth, CI proves the tree before anything ships, and the published artifacts install through the language's native path and, when enabled, Homebrew. Non-goals: nightly or pre-release channels, and package registries beyond those the README lists.

## Requirements

- **dev-R001** — Always: `version:read` is the sole source of truth for the version, and every file derived from it matches before a release is created.
- **dev-R002** — If the release tag already exists locally or on GitHub, then `release:preflight` fails and names the tag.
- **dev-R003** — If the worktree is dirty, the branch is not `main`, or `main` is behind `origin/main`, then `release:preflight` fails and lists every problem it found.
- **dev-R004** — If `cliff.toml` names a different `owner/repo` than the `origin` remote, then `release:preflight` fails.
- **dev-R005** — When `release:commit` runs, only the files `version:files` reports may be modified; any other change aborts the release.
- **dev-R006** — Always: the GitHub release is created from the head of `origin/main`, and GitHub creates the tag; no tag is pushed separately.
- **dev-R007** — When a release is published, the build workflow attaches one `<name>-<target>-v<version>.tgz` per target triple, each with a `.sha256` sidecar.
- **dev-R008** — Always: every `uses:` in `.github/workflows/` is pinned to a commit SHA with a trailing version comment, and `mise run ci-audit` passes.
- **dev-R009** — Always: CI reports lint and test failures as file-and-line annotations on the diff.
- **dev-R010** — While the repository is flagged as a template, CI jobs are skipped.
- **dev-R011** — When the `HOMEBREW_TAP` repository variable is `true`, publishing a release rewrites the formula in the tap from the release's checksums.
- **dev-R012** — Always: `mise run release:rehearse` runs every read-only step of the release and writes nothing.
- **dev-R013** — Always: every commit passes the prek hooks on its staged files, and a subject git-cliff cannot parse is rejected at commit time.
- **dev-R014** — When a branch merges through `wt merge`, one gate runs after the rebase, `release:check` when the target is the default branch and `check` otherwise; a failure aborts the merge.
