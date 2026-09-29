# Engineering harness

## Goals

The harness keeps the generator honest: one local gate, tests that prove the spec rather than the plumbing, a statistical baseline for the ecommerce scenario, and throughput numbers that show whether parallel generation pays off.

## Requirements

- **dev-R015** — `mise run check` runs formatting, clippy with pedantic lints as errors, every test suite, and the hook suite, and CI runs the same task.
- **dev-R016** — An integration test runs the binary twice per scenario with the same seed and flags and asserts every output file byte-matches.
- **dev-R017** — An integration test checks every primary key and foreign key in every output file of every scenario, including nullable foreign keys.
- **dev-R018** — A parity test compares ecommerce summary statistics from the Rust binary against a checked-in fixture captured from the Go reference, each within a tolerance recorded beside the fixture.
- **dev-R019** — The parity fixture covers row counts per entity per year, persona share of orders, mean items per order, mean order total per store, sparrow rate, and guild rank cohort sizes.
- **dev-R020** — `mise run bench` reports rows per second and peak memory for each scenario at a fixed seed, scale, and duration, and at worker counts 1 and all cores.
- ~~dev-R021~~ — retired: the Go reference was deleted once ecommerce parity was proven.
- **dev-R022** — SaaS metric tests compute MRR, ARR, MRR movements, cohort retention, funnel conversion, and CAC from the output files and assert the invariants in `sp-saas-product.md` and `gm-go-to-market.md`.
- **dev-R031** — `mise run bench` includes an ecommerce run at scale 100, large enough that generation and writing dominate process startup.
- **dev-R032** — `mise run profile` builds a symbolized release binary outside `target/release` and records a CPU profile of a chosen invocation.
