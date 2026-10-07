# Rowing Machine

Deterministic synthetic data generator for SQL training and analytics demos. It simulates an ecommerce shop (the data factory for Queria, a retro-RPG SQL trainer), a B2B SaaS company, and a travel network, writing relational files from a single seed. Written in Rust and distributed as a single binary.

## Run work through tasks

[mise](https://mise.jdx.dev) owns the toolchain and the task list. `mise tasks` lists every task; `mise tasks info <task>` prints one task's definition. Prefer a task over the command it wraps, and add a task rather than running a one-off. CI runs these same tasks, so a task definition is the only place a check lives. `mise run check` is the full local gate.

Task scripts live in `mise-tasks/`, grouped into directories that become the `version:`, `release:`, `repo:`, and `ci-audit:` prefixes. Each is plain bash that runs standalone, and they compose by calling each other through those paths. `depends` runs in parallel, so order-sensitive steps belong in a sequential `run` array. `deny_net` and `deny_write` are honored only on TOML tasks.

Rust comes from rustup through `rust-toolchain.toml`, never from mise. For a single test, `cargo test <name>` is fine.

## Rust conventions

- One package: `src/lib.rs` holds everything testable, `src/main.rs` only parses and runs.
- Clippy pedantic lints fail CI. Allow an individual lint at the crate root when it fights the code; never drop the group.
- Errors are `anyhow::Result`, with `.context()` naming what was being attempted. User-facing errors name the flag, value, and fix (`cl-R011`).
- Money is `i64` cents everywhere (`R002`). Floats appear only inside a rate multiplication, rounded straight back to cents.
- Tests sit in `#[cfg(test)]` modules beside the code; integration tests that drive the binary live in `tests/`.
- Add a crate only when the work needs it; `docs/architecture.md` lists the intended picks.

## Determinism is load-bearing

- Same seed and flags must give byte-identical files (`R001`) at every worker count. Every random draw goes through a stream from `engine::stream` (`sm-R010`, `sm-R011`); never use `thread_rng`, system entropy, the clock, or `HashMap` iteration order in anything that reaches output.
- Generate each work unit as a pure function of the seed and its indices (`sm-R030`). Compute cross-unit facts from emitted rows after generation (`sm-R032`).
- Never reorder stores or other indexed catalog entries; the index feeds stream derivation.
- Scenarios never branch on format or worker count; writers never branch on scenario or theme.

## Scenarios own logic, themes own words and numbers

- A scenario owns its generic entities, keys, and simulation logic, and declares every name kind, label set, catalog, and parameter it reads.
- A theme owns table and column names (`[schema.<scenario>]`), name generators, labels, catalogs, and parameter values (`[params.<scenario>]`).
- Scenario logic may branch on parameter values, never on which theme is loaded. When a theme needs different behavior, add a declared parameter with a default and range.
- Bundled themes in `themes/` never contain real brand names (`th-R022`). A branded variant is a private TOML file loaded with `--theme <path>` and is never committed to `main`.

## Releases are human-gated

Never run `release`, `release:push`, or `release:create`. They push commits and create public GitHub releases behind mise `confirm` gates. `release:rehearse` is the dry run: run it when the project looks ready, report what it says, and stop. `version:read` reports where the version is declared; never hand-edit the files `version:files` lists.

## Hooks guard commits, tasks guard merges

prek runs file hygiene and rustfmt on every commit (staged files only) and rejects commit subjects git-cliff cannot parse. `wt merge` runs one gate after the rebase: `release:check` into `main`, `check` into any other branch. Never commit with `--no-verify`; fix what the hook reports. Every `uses:` under `.github/workflows/` stays SHA-pinned, and workflow changes pass `mise run ci-audit`.

## Planning and landing work

Work is planned in the [Rowing Machine](https://linear.app/supermodellabs/project/rowing-machine-a494a26c0af4) Linear project (`supermodellabs` workspace, team `WBG`); `mise.toml` sets `LINEAR_CLI_PROFILE`. `SPEC.md` and `specs/` hold requirements with stable IDs.

- Move an issue to In Progress when its worktree starts (`wt switch -c <branch>`). Branch names describe the work, never the issue key.
- Land on `main` with `wt merge`; there is no PR, so the landing commit carries `Closes WBG-<n>`.
- Agents never push `main`. The owner pushes after the local merge gate passes.
- In an issue, link the spec file on `main` (`https://github.com/gwenwindflower/rowing-machine/blob/main/specs/<file>.md`) rather than citing a bare requirement ID.
- Changing or retiring a shipped requirement gets an ADR in `docs/adr/` unless the owner says otherwise.

## Docs

- `docs/architecture.md` — module layout, the contracts between modules, and dependencies
- `docs/scenarios/README.md` — how scenarios and themes fit together, bundled themes, and shared run flags
- `docs/scenarios/ecommerce.md` — ecommerce entities, order drivers, personas, catalog structure, parameters, and theme slots
- `docs/scenarios/saas.md` — SaaS entities, marketing, sales, subscriptions, usage, theme slots, and example queries
- `docs/scenarios/travel.md` — travel entities, network, schedule, operations, demand, parameters, and theme slots
- `docs/themes.md` — theme file schema, name assignment, and validation
- `docs/output-schema.md` — column reference for every output file
- `docs/performance.md` — benchmark method and recorded results
- `docs/adr/` — decision records for changes to shipped requirements
