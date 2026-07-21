# CLI Surface — Spec

Owns the flag surface, defaults, validation rules, and user-facing output. Lives in `cmd/rowing-machine/main.go`, wired through Cobra.

The CLI is the primary contract with users and the Queria training app. Flags MUST behave predictably across the documented domain — see `R001` (seed determinism), and per-flag requirements below.

## Vocabulary

- **Mode** — a top-level run shape (default `generate`; future modes might include preview, validate, etc. — no current need).
- **Pre-calculation** — *(planned)* a phase before generation where the tool reports the calibrated duration for a `--target-rows` request.

## Requirements

### Current flags

- **cl-R001 `--years <int>`.** Number of years to simulate (365 days each). Default `4`.
- **cl-R002 `--scale <int>`.** Customer pool multiplier applied to each store's `TAM_base`. Default `100`.
- **cl-R003 `--seed <int64>`.** Random seed. `0` (default) means generate a random seed and print it to stdout. Any non-zero value pins all randomness.
- **cl-R004 `--start-date <YYYY-MM-DD>`.** Simulation epoch (day index 0). Default `2023-01-01`, producing data through 2026 with the default duration. Invalid date format MUST return a clear error before any work starts.
- **cl-R005 `--output-dir <path>`.** Output directory. Default `./factory-output`. Created if it doesn't exist.
- **cl-R006 `--pre <string>`.** Filename prefix for each entity file. Default `raw`.
- **cl-R007 `--quiet`.** Suppress progress bar and seed-print output. Default `false`. Errors still go to stderr.

### Help and error UX (Supermodel Labs DX)

- **cl-R010 Verbose help.** `--help` output MUST explain each flag with enough context that an agent can pick the right invocation without external docs. Defaults shown, examples for any non-obvious flag.
- **cl-R011 Actionable errors.** Validation errors (bad date, unwritable output dir, conflicting flags) MUST name the offending flag and value and suggest a fix. No bare stack traces from `cobra.Command.RunE`.
- **cl-R012 Positive generation sizes.** `--years` and `--scale` MUST each be greater than zero. Invalid values MUST fail before output files are created and name the offending flag and value.

### Planned — Phase 1

- **cl-R020 `--target-rows <int>`.** Calibrate simulation duration to produce approximately this many `orders` rows. Implementation auto-computes the necessary `--years` by sampling output density at the chosen scale. While calibrating, the CLI MUST clearly indicate pre-calculation mode is active and switch over to a normal progress display once generation starts. Mutually exclusive with `--years` (passing both is an error).
- **cl-R021 `--format <csv|jsonl|parquet>`.** Output format. Default `csv`. Behavior per `op-R020`, `op-R021`.
- **cl-R022 `--compress`.** Enable format-appropriate compression. Behavior per `op-R022`. Error if combined with `--format csv` (or alternatively a silent no-op — decided at implementation time, but MUST NOT silently produce uncompressed parquet when the user asked for compression).

### Planned — Phase 2

- **cl-R030 `--workers <int>`.** Number of parallel workers. `0` or `1` (default `1`) runs serially. Higher values fan worker goroutines across the simulation timeline. Output MUST remain byte-identical to the serial run for the same seed (see `R001` and the parallelism contract added under `sm-R030+` when Phase 2 lands).

### Planned — Phase 3

- **cl-R040 `--theme <name>`.** Selects a bundled or path-loaded theme. Default — TBD when Phase 3 plans (likely `default` for plain ecommerce, with `fantasy_rpg` as the bundled flavored theme).

### Planned — Phase 4

- **cl-R050 `--messy [level]`.** Enable mess injection. The exact knob shape (single boolean vs. graded levels vs. independent toggles) is intentionally underspecified — Phase 4 will refine it. The spec contract MUST be that the same seed + the same `--messy` setting produces byte-identical output.

## Out of scope for this domain

- Output file shape → `op-output.md`.
- Simulation behavior driven by the flag values → `sm-simulation.md`.
- Static data tables — flags do not alter the catalog roster directly → `dt-catalog.md`.
