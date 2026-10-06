# CLI

## Goals

The CLI is the contract with people, lesson pipelines, and agents. Flags behave predictably, defaults produce a useful dataset with no arguments, and every error names the flag, the value, and the fix. Help text is complete enough that an agent can pick the right invocation without other docs.

## Requirements

### Generation flags

- **cl-R001** — `--years <int>` sets the number of 365-day years to simulate; default `4`.
- **cl-R002** — `--scale <int>` multiplies each scenario's base population (store TAM for ecommerce, addressable accounts for SaaS); default `100`.
- **cl-R003** — `--seed <u64>` pins all randomness; `0`, the default, picks a random seed and prints it (`R005`).
- **cl-R004** — `--start-date <YYYY-MM-DD>` sets day index 0, default `2023-01-01`; an invalid date fails before any work starts.
- **cl-R005** — `--output-dir <path>` sets the output directory, default `./factory-output`, creating it if missing.
- **cl-R006** — `--pre <string>` sets the entity file prefix; default `raw`.
- **cl-R007** — `--quiet` suppresses progress and the seed print; errors still go to stderr.

### Help and errors

- **cl-R010** — `--help` explains every flag with its default, and shows an example for any flag whose effect is not obvious from its name.
- **cl-R011** — Validation errors name the offending flag and value and suggest a fix; the binary never prints a panic or backtrace for bad input.
- **cl-R012** — `--years` and `--scale` must be greater than zero, checked before any output file is created.

### Output controls

- **cl-R020** — `--target-rows <int>` picks the duration that produces about that many rows of the scenario's calibration entity (`orders` for ecommerce), showing a calibration indicator before normal progress starts.
- **cl-R023** — If `--target-rows` and `--years` are both passed, then the binary fails naming both flags.
- **cl-R021** — `--format <csv|jsonl|parquet>` selects the output format; default `csv`.
- **cl-R022** — If `--compress` is combined with `--format csv`, then the binary fails and suggests `jsonl` or `parquet`.

### Parallelism

- **cl-R030** — `--workers <int>` sets the number of worker threads; the default is the number of available cores, `1` runs serially, and `0` is rejected.

### Scenarios and themes

- **cl-R040** — `--theme <name>` selects a bundled theme by name or a theme file by path; default `plain`, and the theme must be compatible with the scenario (`th-R017`).
- **cl-R041** — `--scenario <ecommerce|saas>` selects the business model; default `ecommerce`.
- **cl-R042** — `rowing-machine themes` lists every bundled theme with a one-line description and the scenarios it is compatible with.
- **cl-R043** — `--param <name>=<value>` overrides one of the selected scenario's parameters for the run and can repeat; an unknown name or out-of-range value fails before output, naming `--param`, the scenario's parameters, or the valid range.
