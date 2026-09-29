# Performance

Run `mise run bench` to compare ecommerce CSV generation with one worker and all available cores at seed 42, scale 10, and four 365-day years starting 2023-01-01. Available cores come from Rust's `std::thread::available_parallelism`. Criterion reports emitted rows per second across all seven entities. Each iteration creates a fresh dataset; temporary-directory setup and cleanup are excluded from generation timing.

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
