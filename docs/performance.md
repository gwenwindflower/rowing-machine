# Performance

Run `mise run bench` to measure serial ecommerce CSV generation at seed 42, scale 10, and four 365-day years starting 2023-01-01. Criterion reports emitted rows per second across all seven entities. Each iteration creates a fresh dataset; temporary-directory setup and cleanup are excluded from generation timing.

The task compiles first, then runs the benchmark executable under the platform's `time` command. Maximum resident memory covers the entire Criterion process, including initial row counting, warmup, and repeated samples; compilation is excluded. It is not a measurement of one isolated CLI invocation.

## Serial baseline

Measured on macOS arm64 on 2026-09-28 with 10 Criterion samples:

| Measure | Result |
| --- | --- |
| Emitted rows | 417,590 |
| Generation time | 582 ms |
| Throughput | 717,380 rows/sec |
| Throughput interval | 712,010–721,700 rows/sec |
| Maximum resident memory | 75,235,328 bytes (71.75 MiB) |

The engine retains customer pools, customer order counts, and primary keys for duplicate detection. Full order, item, and sparrow rows stream through one market-day at a time. Primary-key storage grows with emitted row count. The sparrow stage regenerates order decisions after customer ranks are finalized, trading CPU work for avoiding an in-memory copy of the orders.

Worker-count comparisons belong to the parallel scheduler phase; the current engine executes serially.
