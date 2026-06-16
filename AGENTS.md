# Rowing Machine

Deterministic synthetic data generator for **Queria**, a retro-RPG SQL trainer. Simulates the Arcanum Collective — a mage guild running guild halls across fantasy towns — producing seven relational CSV files (`stores`, `customers`, `orders`, `items`, `products`, `supplies`, `sparrows`).

This project uses **SPOT** — see `~/.agents/rules/projects.md` for the system. Specs hold *what*, DONE.md holds *why*, code holds *how*.

## Commands

```bash
go build ./cmd/rowing-machine          # build
go run ./cmd/rowing-machine             # run
go test ./...                           # all tests
go test ./internal/simulation/ -run X   # single test
go vet ./...                            # static analysis
```

## Conventions

- Module path: `rowing-machine` (matches `go.mod`).
- Lowercase single-word package names.
- Errors return `error`, wrap with `fmt.Errorf("context: %w", err)`.
- Table-driven tests in `_test.go` alongside source.
- No `init()` — explicit initialization only.
- Enums: `type X int` with `iota` constants and a `String()` method.
- Money: `int64` cents end-to-end (see `R002`). Floats only inside tax-rate multiplication, immediately rounded back to cents.

## Project layout

```text
cmd/rowing-machine/main.go      CLI entrypoint (Cobra)
internal/catalog/               Static data: products, supplies, name pool
internal/market/                Per-store simulation: customer pool, day loop, p_buy
internal/models/                Core types: Customer/Persona, Order, Store, Sparrow, Supply, Item, Season, UUID
internal/simulation/            Orchestrator: Config, Run(), day precomputation, curves, progress, integration tests
internal/output/                OutputWriter interface and per-format implementations
specs/                          Durable per-domain requirements
docs/                           Implementation references (formulas, static data tables, schemas)
```

## Working files

- `SPEC.md` — project contract, vocabulary, project-scope requirements, domain spec index.
- `TODO.md` — Phases → Objectives → Tasks for in-flight work, with `**Requirements**:` lines linking to spec IDs.
- `DONE.md` — shipped Phases with rationale.

## Specs (the *what*)

- `specs/sm-simulation.md` — formulas, curves, RNG discipline, day loop, persona behavior
- `specs/dt-catalog.md` — static data: guild halls, products, supplies, persona roster, name pool
- `specs/op-output.md` — file layout, schemas, format and compression contracts
- `specs/cl-cli.md` — flag surface (current + planned)

## Docs (the *how*, for orientation)

- `docs/simulation.md` — formula reference and order-generation flow
- `docs/static-data.md` — hardcoded value tables (mirrors `internal/catalog/` for quick lookup)
- `docs/output-schema.md` — canonical column reference for the seven output files

## Watchouts

- **Determinism is load-bearing.** Same seed in MUST give byte-identical output (`R001`). Any new randomness MUST flow through an existing PRNG stream, not `crypto/rand` or `time.Now()`.
- **Don't change guild hall ordering.** Hall index is the PRNG offset for per-market seeding (`sm-R011`, `dt-R004`). Reordering breaks seed stability for every downstream user.
- **`market` defines its own `DayInfo`.** Mirrors `simulation.DayState` to break a `simulation → market → simulation` import cycle. The orchestrator converts between them; don't collapse them.
