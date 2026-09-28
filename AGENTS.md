# Rowing Machine

Deterministic synthetic data generator for SQL training and analytics demos. It simulates an ecommerce shop (the data factory for Queria, a retro-RPG SQL trainer) and a B2B SaaS company, writing relational files from a single seed. Written in Rust and distributed as a single binary.

The project is mid-rewrite from Go. `TODO.md` holds the Phases, `docs/architecture.md` the module seams and build lanes, and `go-reference/` the Go implementation to port from. Never edit `go-reference/` or add it to `mise run check` or CI (`dev-R021`). Read it for behavior, and build and run it only to capture the parity fixture (`dev-R018`).

## Run work through tasks

[mise](https://mise.jdx.dev) owns the toolchain and the task list. `mise tasks` lists every task with its description; `mise tasks info <task>` prints one task's definition. Prefer a task over the command it wraps, and add a task rather than running a one-off. CI runs these same tasks, so a task definition is the only place a check lives. `mise run check` is the full local gate.

Task scripts live in `mise-tasks/`, grouped into directories that become the `version:`, `release:`, and `repo:` prefixes. Each is plain bash that runs standalone, and they compose by calling each other through those paths. `depends` runs in parallel, so order-sensitive steps belong in a sequential `run` array. `deny_net` and `deny_write` are honored only on TOML tasks.

Rust comes from rustup through `rust-toolchain.toml`, never from mise. For a single test, `cargo test <name>` is fine.

## Rust conventions

- One package: `src/lib.rs` holds everything testable, `src/main.rs` only parses and runs.
- Clippy pedantic lints fail CI. Allow an individual lint at the crate root when it fights the code; never drop the group.
- Errors are `anyhow::Result`, with `.context()` naming what was being attempted. User-facing errors name the flag, value, and fix (`cl-R011`).
- Money is `i64` cents everywhere (`R002`). Floats appear only inside a rate multiplication, rounded straight back to cents.
- Tests sit in `#[cfg(test)]` modules beside the code; integration tests that drive the binary live in `tests/`.
- Add a crate only in the Phase that needs it; `docs/architecture.md` lists the intended picks.

## Determinism is load-bearing

- Same seed and flags must give byte-identical files (`R001`) at every worker count. Every random draw goes through a stream from `engine::stream` (`sm-R010`, `sm-R011`); never use `thread_rng`, system entropy, the clock, or `HashMap` iteration order in anything that reaches output.
- Generate each work unit as a pure function of the seed and its indices (`sm-R030`). Compute cross-unit facts from emitted rows after generation (`sm-R032`).
- Never reorder guild halls or other indexed catalog entries; the index feeds stream derivation (`dt-R004`).
- Scenarios never branch on format or thread count; writers never branch on scenario or theme.

## Releases are human-gated

Never run `release`, `release:push`, or `release:create`. They push commits and create public GitHub releases behind mise `confirm` gates. `release:rehearse` is the dry run: run it when the project looks ready, report what it says, and stop. `version:read` reports the version's single source of truth; never hand-edit the files it derives.

## Hooks guard commits, tasks guard merges

prek runs file hygiene and rustfmt on every commit (staged files only) and rejects commit subjects git-cliff cannot parse. `wt merge` runs one gate after the rebase: `release:check` into `main`, `check` into any other branch, so Phase branches folding into `feat/rust-rewrite` run `check`. Never commit with `--no-verify`; fix what the hook reports. Every `uses:` under `.github/workflows/` stays SHA-pinned, and workflow changes pass `mise run ci-audit`.

## Planning

This project uses SPOT with the repo plan: `SPEC.md` and `specs/` hold requirements with stable IDs, `TODO.md` holds active Phases, `DONE.md` is the ledger, and `docs/adr/` records reversals of shipped requirements. Commit bodies carry `Completes <Objective> in Phase N` and `Closes Phase N` after any body bullets and before trailers.

Until Phase 13 merges it into `main`, the rewrite lives on `feat/rust-rewrite`. Each Phase runs on its own worktree branched from it (`wt switch --create <branch> --base feat/rust-rewrite`) and folds back with `wt merge --no-squash feat/rust-rewrite`. `docs/architecture.md` lists which Phases can run as parallel sessions.

## Docs

- `docs/architecture.md` — module layout, the contracts between modules, dependency picks, the Go-to-Rust porting map, and build lanes
- `docs/simulation.md` — ecommerce formula reference and order generation flow
- `docs/static-data.md` — ecommerce catalog tables
- `docs/output-schema.md` — column reference for every output file
- `docs/adr/` — decision records for changes to shipped requirements
- `docs/bootstrap.md` — remaining template provisioning steps, deleted in Phase 13
