# Performance

Run `mise run bench` to compare ecommerce and SaaS CSV generation at scales 10 and 100. Scale 100 is the CLI default. Every case uses seed 42, four 365-day years starting 2023-01-01, and one worker versus all available cores. Available workers come from Rust's `std::thread::available_parallelism`. Criterion reports emitted rows per second across every entity in the scenario. Each iteration creates a fresh dataset; temporary-directory setup and cleanup are excluded from generation timing.

The task compiles first, then runs each scenario, scale, and worker setting in a separate benchmark process under the platform's `time` command. Maximum resident memory covers that Criterion process, including serial row counting for its selected scenario and scale, warmup, and repeated samples; compilation is excluded. It is not a measurement of one isolated CLI invocation. Internal `ROWING_BENCH_SCENARIO` and `ROWING_BENCH_SCALE` filters keep unselected row-count runs out of each process; direct `cargo bench --bench scenarios` runs all cases.

After measurement, the task reads Criterion's fresh scale-100 estimates and recorded row count, prints PASS/FAIL for the 2-million serial rows/sec requirement and the greater-than-2× worker speedup requirement, and exits unsuccessfully if either applicable requirement fails. The speedup check is skipped when Rust reports fewer than eight available workers. Use `mise run bench -- --quick` for an exploratory run; full samples provide stronger evidence on a quiet machine. Arguments that skip measurement cannot satisfy the fresh-estimate check.

The engine retains customer pools, customer order counts, and typed primary keys for duplicate detection. Primary-key storage grows with emitted row count. The sparrow stage regenerates order decisions after customer ranks are finalized, trading CPU work for avoiding an in-memory copy of the orders.

Workers validate and encode their generated units. A bounded handoff delivers completed units to one writer in declared order while the next units generate. Scenario observation follows declared order, and stage completion waits for preceding writes. With one worker, generation is serial and the handoff holds one completed unit.

## SaaS product usage

Measured before sales pipeline integration with `mise run bench` on macOS arm64 with ten available workers on 2026-09-29. Each case uses ten Criterion samples and includes marketing, revenue, sessions, and events:

| Scale | Rows | Workers | Time estimate | Throughput | Maximum resident memory |
| --- | --- | --- | --- | --- | --- |
| 10 | 1,886,641 | 1 | 562.42 ms | 3.3545 million rows/sec | 811,204,608 bytes |
| 10 | 1,886,641 | 10 | 243.36 ms | 7.7526 million rows/sec | 1,089,634,304 bytes |
| 100 | 20,488,914 | 1 | 5.9805 s | 3.4260 million rows/sec | 1,717,633,024 bytes |
| 100 | 20,488,914 | 10 | 2.8140 s | 7.2810 million rows/sec | 2,094,284,800 bytes |

At the default scale of 100, the serial 95% time interval is 5.9034–6.0723 seconds and the ten-worker interval is 2.7793–2.8457 seconds. The point estimates give a 2.13× speedup. Memory includes Criterion's repeated runs and serial counting pass; the engine retains primary keys for all emitted rows and buffers account-sized generation units. These figures are not isolated CLI peak-memory measurements.

The same run passed the ecommerce performance gates: 2,337,907 serial rows/sec and a 3.05× all-core speedup at scale 100.

## Throughput proof

Measured before SaaS product usage with `mise run bench` on macOS arm64 (Mac17,3, ten available workers) on 2026-09-29, using ten Criterion samples for each case:

| Scenario and scale | Workers | Time estimate | Throughput | Maximum resident memory |
| --- | --- | --- | --- | --- |
| Ecommerce, 10 | 1 | 250.35 ms | 1.6680 million rows/sec | 93,405,184 bytes |
| Ecommerce, 10 | 10 | 120.01 ms | 3.4796 million rows/sec | 161,611,776 bytes |
| Ecommerce, 100 | 1 | 1.7075 s | 2.4352 million rows/sec | 315,359,232 bytes |
| Ecommerce, 100 | 10 | 628.72 ms | 6.6137 million rows/sec | 358,465,536 bytes |
| SaaS, 10 | 1 | 39.830 ms | 169,920 rows/sec | 293,715,968 bytes |
| SaaS, 10 | 10 | 34.256 ms | 197,570 rows/sec | 278,544,384 bytes |

Ecommerce emits 417,590 rows at scale 10 and 4,158,194 at scale 100; SaaS emits 6,768 at scale 10. For scale-100 ecommerce, the serial 95% time interval is 1.6803–1.7385 seconds, corresponding to 2.3918–2.4746 million rows/sec. The ten-worker interval is 618.83–639.91 milliseconds, corresponding to 6.4981–6.7194 million rows/sec. The point estimates give a 2.7159× speedup, meeting `sm-R035`; serial throughput exceeds two million rows/sec, meeting `sm-R036`. The task reported PASS for both requirements.

## Historical serial baseline

Measured on macOS arm64 on 2026-09-28 with 10 Criterion samples:

| Measure | Result |
| --- | --- |
| Emitted rows | 417,590 |
| Generation time | 582 ms |
| Throughput | 717,380 rows/sec |
| Throughput interval | 712,010–721,700 rows/sec |
| Maximum resident memory | 75,235,328 bytes (71.75 MiB) |

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

The initial parallel-output profile contains 576 weighted samples on the writer thread: 269 (46.7%) wait for the next unit in the receive channel, and 297 (51.6%) are inside `write_prepared`, including 106 in the write syscall. Hashing and growing primary-key sets account for most remaining writer work. The main thread has 91 of 621 weighted samples in bounded sends, showing occasional backpressure. The writer's substantial idle time supports keeping one file-writing thread; distributing entity files across more threads would not address the dominant generation and encoding work. These are sampled stack weights, including off-CPU waits, rather than CPU-time percentages.
