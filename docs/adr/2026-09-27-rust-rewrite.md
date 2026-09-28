---
name: Rewrite in Rust with scenarios and themes
date: 2026-09-27
requirements: [R001, R003, sm-R010, sm-R011, sm-R012, sm-R014, dt-R014, dt-R015, op-R002, op-R007, op-R018, op-R022, cl-R003, cl-R030, cl-R040, cl-R050]
status: accepted
---

# Rewrite in Rust with scenarios and themes

## Context

Rowing Machine shipped as a Go binary generating one ecommerce dataset. The next body of work is a much larger SaaS simulation (product events, subscriptions, marketing, sales) where row volume and per-entity state make raw throughput and a strict parallelism story matter most. Several shipped requirements are written in Go terms: `sm-R010` required `math/rand/v2` PCG streams, `sm-R011` pinned the seed formula `seed + marketIndex + 1`, `sm-R014` required a Go package split to avoid an import cycle, and `R003` required `go build` to produce the binary. `op-R002` fixed the output at exactly seven files, and `dt-R014`/`dt-R015` fixed customer names to a JSON name pool. The project also adopts the `gwenwindflower/_tool` template's mise, prek, CI, and release pipeline.

## Decision

Rewrite the tool in Rust as a single crate, keeping the Go code in `go-reference/` as read-only reference until ecommerce parity is proven. Stream derivation becomes a mix of seed, stream name, and indices (`sm-R010`–`sm-R012` edited), `sm-R014` retires, and `sm-R030`–`sm-R034` define a parallel-safe engine from the start. The engine hosts multiple scenarios (`sm-R025`, `op-R002` edited, `cl-R041`–`cl-R043` added) with themes as naming packs usable with any scenario whose name kinds they cover (`th-R009`–`th-R018`, `dt-R014`/`dt-R015` retired); nullable columns are allowed where a schema declares them (`op-R018` edited); `--compress` uses Parquet's internal zstd codec (`op-R022` edited); `--workers` defaults to all cores (`cl-R030` edited); messy mode (`cl-R050`) moves to the Backlog.

## Alternatives considered

### Option A: Keep Go and add the SaaS scenario

Least churn and the codebase already works. The SaaS scenario would still need the engine refactor for scenarios and day-independent streams, which touches most of the Go code anyway, and the rewrite is also a deliberate Rust learning investment.

### Option B: Byte-identical parity with the Go output

Would let every existing seeded dataset survive the rewrite. It requires reproducing Go's PCG variant, float formatting, and its ziggurat normal sampler exactly, and it conflicts with the order-independent stream derivation the parallel engine needs. Rejected in favor of statistical parity (`dev-R018`, `dev-R019`).

### Option C: A Cargo workspace with one crate per scenario

Cleaner compile boundaries, but the template's release, version, and binstall tasks assume one package, and the scenario count is small. Modules inside one crate give the same seams.

## Consequences

### Positives

- One engine runs both scenarios, and parallel output is byte-identical to serial by construction.
- The SaaS scenario gets Rust's throughput where the row volume is highest.
- Release, CI, and hooks follow the same template as the other Supermodel tools.

### Negatives

- Every existing seed produces different data after the rewrite; lessons pinned to a Go-era seed must be regenerated or keep a Go-era file.
- Two implementations coexist in the tree until the Go reference is deleted.
