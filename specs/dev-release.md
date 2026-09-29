# Release and repository plumbing

## Goals

Every release is cut from a clean `main` by a human-gated pipeline that a contributor can rehearse without side effects. The version has one source of truth, CI proves the tree before anything ships, the crate publishes to crates.io so `cargo binstall` finds the release archives, and Homebrew installs from the same archives. Non-goals: nightly or pre-release channels, and package registries beyond crates.io and the Homebrew tap.

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
- **dev-R023** — `mise run check`, CI, and merge gates build the optimized binary and compile the packaged crate without publishing it.
- **dev-R024** — If the checkout is dirty, does not match its release tag, or the release lacks any of the four archives and checksums, then crate publication fails before uploading.
- **dev-R025** — The first crate publication runs through a confirmed local task using Cargo's configured credentials.
- **dev-R026** — When the `CRATES_IO_PUBLISHING` repository variable is `true`, publishing a release publishes the crate from CI after the binary uploads, using a short-lived OIDC token in the `release` environment.
- **dev-R027** — Release asset recovery uploads a completed release run's archives to its existing release only after checking the run's workflow, tag, and checksums.
- **dev-R028** — Repository provisioning derives required status checks only from the CI workflow on `main`, never from release jobs.
- **dev-R030** — `mise run check` and CI run each test suite once and never run interactive or pretty-output tasks.
