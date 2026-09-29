# Performance

Run `mise run bench` to compare ecommerce CSV generation with one worker and all available cores at seed 42, scales 10 and 100, and four 365-day years starting 2023-01-01. Available cores come from Rust's `std::thread::available_parallelism`. Criterion reports emitted rows per second across all seven entities. Each iteration creates a fresh dataset; temporary-directory setup and cleanup are excluded from generation timing.

The task compiles first, then runs each worker setting in a separate benchmark process under the platform's `time` command. Maximum resident memory covers that Criterion process, including serial row counting, warmup, and repeated samples; compilation is excluded. It is not a measurement of one isolated CLI invocation.

## Serial baseline

Measured on macOS arm64 on 2026-09-28 with 10 Criterion samples:

| Measure | Result |
| --- | --- |
| Emitted rows | 417,590 |
| Generation time | 582 ms |
| Throughput | 717,380 rows/sec |
| Throughput interval | 712,010–721,700 rows/sec |
| Maximum resident memory | 75,235,328 bytes (71.75 MiB) |

The engine retains customer pools, customer order counts, and primary keys for duplicate detection. Primary-key storage grows with emitted row count. The sparrow stage regenerates order decisions after customer ranks are finalized, trading CPU work for avoiding an in-memory copy of the orders.

The parallel scheduler buffers rows for at most four work units per worker in each batch and writes them in declared unit order. Output writing and observation remain serial, with a barrier between stages. A single worker streams one unit at a time.

## Worker comparison

Measured on macOS arm64 (Mac17,3, 10 available cores) on 2026-09-29 with 10 Criterion samples per worker setting and 417,590 emitted rows:

| Measure | 1 worker | 10 workers |
| --- | --- | --- |
| Generation time | 914.06 ms | 929.78 ms |
| Throughput | 456,850 rows/sec | 449,130 rows/sec |
| Throughput interval | 446,370–467,290 rows/sec | 408,440–487,990 rows/sec |
| Maximum resident memory | 74,252,288 bytes (70.81 MiB) | 77,873,152 bytes (74.27 MiB) |

The intervals overlap, so this run does not demonstrate a throughput gain from parallel generation. Criterion flagged two high outliers in the 10-worker samples. These are end-to-end generation and CSV-writing measurements, with serial output work included; they do not isolate generation CPU time. The earlier serial baseline is historical and was measured separately.

## Scale-100 baseline

The scale-100 baseline emits 4,158,194 rows. Single CLI trials of the pre-optimization release binary on the same machine, with seed 42 and four years, measured:

| Measure | 1 worker | 10 workers |
| --- | --- | --- |
| Wall time | 10.34 s | 9.92 s |
| User CPU time | 9.88 s | 10.95 s |
| Throughput | 402,146 rows/sec | 419,173 rows/sec |
| Maximum resident memory | 529,104,896 bytes | 542,031,872 bytes |

These are single CLI samples, not Criterion estimates. Their near-equal wall times and near-single-core CPU use support the serial-output bottleneck; they do not establish a statistically significant worker speedup.

## Serial regression investigation

The regression narrows to `c58606e` (`feat(output): stream deterministic JSONL files`). Its parent, `9c16bdd`, passes the CSV fields already formatted for primary-key validation directly to the writer. The JSONL change passes the typed row to `EntityWriter::write_row` instead, which formats every CSV field a second time. This happens before the worker-pool commit, `90da8ad`.

Adjacent revisions were extracted with `git archive`, built with `cargo build --release --locked` in a shared scratch target, and run serially on the same macOS arm64 machine on 2026-09-29. Both used seed 42, scale 10, four years, CSV, quiet output, and 417,590 emitted rows. `/usr/bin/time -l` measured the CLI process; these figures are separate from Criterion's intervals above.

| Revision | First wall time | Repeat wall time | Repeat user time | Repeat instructions |
| --- | --- | --- | --- | --- |
| `9c16bdd` | 0.78 s | 0.58 s | 0.54 s | 10.96 billion |
| `c58606e` | 0.99 s | 0.83 s | 0.78 s | 18.42 billion |

The repeated run takes 43% longer and executes 68% more instructions at the JSONL commit. Primary-key string storage and serial output remain additional costs, but the duplicated formatting explains the large serial regression without attributing it to thread-pool overhead.

## CPU profiling

Install [samply](https://github.com/mstange/samply) with `cargo install --locked samply`, then supply normal CLI arguments:

```sh
mise run profile -- --seed 42 --scale 100 --years 4 --workers 10 --quiet --output-dir /tmp/rowing-profile-output
```

The task prints a scratch directory containing an optimized binary with full debug information and the recorded `profile.json.gz`. It preserves the binary and symbols for `samply load`, leaves `target/release` untouched, and does not open a browser or upload the profile. Remove the printed directory when finished inspecting it. macOS and Linux profiling require host access to sampling facilities; Linux additionally requires permission to use performance events.

The scale-100 command above was verified on macOS arm64 with samply 0.13.1 on 2026-09-29. The resulting profile contained samples from twelve threads. Keep the entire scratch target: macOS stores split debug information in its object files.
