# Performance

`mise run bench` measures CSV throughput and peak memory for ecommerce and SaaS and checks the ecommerce performance requirements, `sm-R035` and `sm-R036`. The travel scenario has no benchmark yet.

## Benchmark method

| Setting | Value |
| --- | --- |
| Cases | ecommerce and saas, each at scale 10 and 100 |
| Run | Seed 42, four 365-day years from 2023-01-01, CSV, `plain` theme |
| Workers | 1 versus all available (`std::thread::available_parallelism`) |
| Samples | 10 Criterion samples, 10-second measurement time |
| Metric | Emitted rows per second, summed across every entity |

- Each iteration writes a fresh dataset; temporary-directory setup and cleanup are outside the timing.
- The task compiles once, then runs each scenario, scale, and worker setting in its own benchmark process under `/usr/bin/time` (`-l` on macOS, `-v` on Linux). The internal `ROWING_BENCH_SCENARIO` and `ROWING_BENCH_SCALE` variables select one case per process; `cargo bench --bench scenarios` runs every case.
- Peak memory covers the whole Criterion process, including a serial row-count run, warmup, and repeated samples. It is not the peak of one CLI invocation.
- After measuring, the task reads Criterion's fresh scale-100 ecommerce estimates and prints PASS or FAIL for each requirement. It exits nonzero on a failure.

| Requirement | Check |
| --- | --- |
| `sm-R036` | One-worker throughput of at least 2 million rows per second |
| `sm-R035` | All-core run more than 2× faster than one worker; skipped with fewer than 8 available workers |

`mise run bench -- --quick` gives an exploratory run. Arguments that skip measurement leave no fresh estimates, so the requirement check fails.

## Recorded results

All results come from a macOS arm64 machine (Mac17,3) with ten available workers, measured on 2026-09-29.

### Ecommerce

| Scale | Rows | Workers | Time estimate | Throughput | Peak memory |
| --- | --- | --- | --- | --- | --- |
| 10 | 417,590 | 1 | 250.35 ms | 1.67 million rows/sec | 93,405,184 bytes |
| 10 | 417,590 | 10 | 120.01 ms | 3.48 million rows/sec | 161,611,776 bytes |
| 100 | 4,158,194 | 1 | 1.7075 s | 2.44 million rows/sec | 315,359,232 bytes |
| 100 | 4,158,194 | 10 | 628.72 ms | 6.61 million rows/sec | 358,465,536 bytes |

At scale 100 the serial 95% interval is 1.6803–1.7385 s and the ten-worker interval is 618.83–639.91 ms, a 2.72× speedup. Both requirements passed. A later run on the same machine and day passed again at 2,337,907 serial rows/sec and a 3.05× speedup.

### SaaS

These figures include marketing, revenue, sessions, and events, and predate the sales pipeline entities.

| Scale | Rows | Workers | Time estimate | Throughput | Peak memory |
| --- | --- | --- | --- | --- | --- |
| 10 | 1,886,641 | 1 | 562.42 ms | 3.35 million rows/sec | 811,204,608 bytes |
| 10 | 1,886,641 | 10 | 243.36 ms | 7.75 million rows/sec | 1,089,634,304 bytes |
| 100 | 20,488,914 | 1 | 5.9805 s | 3.43 million rows/sec | 1,717,633,024 bytes |
| 100 | 20,488,914 | 10 | 2.8140 s | 7.28 million rows/sec | 2,094,284,800 bytes |

At scale 100 the serial 95% interval is 5.9034–6.0723 s and the ten-worker interval is 2.7793–2.8457 s, a 2.13× speedup.

## Where memory goes

- The output thread keeps every primary key for duplicate detection, so key memory grows with row count.
- Ecommerce keeps customer pools and per-customer order counts; travel keeps per-traveller booking counts.
- SaaS buffers one account's rows per stage-2 unit, and the batch size bounds how many units are held at once.
- Encoded payloads are bounded by the batch size and output queue; see [the engine loop](architecture.md#engine-loop).

## CPU profiling

Install [samply](https://github.com/mstange/samply) with `cargo install --locked samply`, then pass normal CLI arguments:

```sh
mise run profile -- --seed 42 --scale 100 --years 4 --workers 10 --quiet --output-dir /tmp/rowing-profile-output
```

The task builds an optimized binary with full debug info in a scratch directory and records `profile.json.gz` there, leaving `target/release` alone. It prints the directory, keeps the binary and symbols for `samply load`, and never opens a browser or uploads the profile. Keep the whole directory while inspecting, since macOS stores split debug info in the object files, and remove it when done. Sampling needs host access on macOS and Linux; Linux also needs permission to use performance events.

A scale-100, ten-worker profile shows the output thread about half idle waiting for units and half writing, with occasional backpressure on the engine thread. Generation and encoding on the workers dominate, which is why one output thread is enough.
