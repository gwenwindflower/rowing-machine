# Ecommerce reference fixture

`go-reference-stats.json` records the Go reference at seed 42, scale 10, four 365-day years, starting 2023-01-01. Capture it from the repository root:

```sh
mise run parity-capture
```

The capture task builds and runs the reference and saves its CSV output in `target/go-parity/output`. It uses a Go build overlay to run `tests/support/go_personas.go`, which reconstructs the same customer pools through the reference's public market API and writes an auxiliary customer UUID/persona mapping. The overlay leaves `go-reference/` untouched. Persona cannot be recovered exactly from the seven output tables: several personas produce overlapping baskets. Rust parity tests obtain the equivalent mapping from the deterministic customer pools. Neither normal checks nor CI build Go.

The Rust statistics reader joins items to their order year, assigns customers to their first order year, groups stores by opening year, and groups sparrows by sent year. Products and supplies have no timestamp and use the `static` bucket. Order totals are cents and store names identify the corresponding markets across independently generated UUIDs. Persona shares and sparrow rate divide by the number of orders; guild cohorts count emitted customers.

Each metric records its reference value, absolute tolerance, and relative tolerance. A candidate passes when its absolute difference is at most `absolute_tolerance + relative_tolerance * abs(value)`. Missing metrics count as zero, including years with no emitted rows; unexpected nonzero metrics fail.

Tolerances are chosen before comparing Rust output. Exact catalog and store counts have zero tolerance. At scale 10, each market has only 80–140 customers; replacing the random streams resamples their persistent preferences as well as daily decisions, so daily order counts are not independent samples. The 20% row-count bounds (plus 20 rows) accommodate that customer clustering. First-order customer counts use 10% plus five customers; lifetime rank cohorts use 5% plus two. Mean basket size uses 10%; mean order totals use 25% because rare expensive products and persistent basket preferences inflate variability. Sparrow rate uses five percentage points. Persona shares use the smaller of five percentage points and `0.001 + 0.25 * reference_share`; the reference-dependent bound catches missing rare personas that a uniform five-point bound would miss. These are conservative regression bounds, not confidence intervals, and do not establish exact equivalence of probability distributions. Formula and invariant tests provide the tighter checks on individual behaviors.
