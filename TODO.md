# Rowing Machine Task List

## Phase 1

- [ ] Implement `--target-rows` flag to create a target count of data, requires auto-calculating simulation duration, should clearly communicate when it's in pre-calculation mode, and when it ticks over to actual data generation
- [ ] Add multiple formats for output: `--format jsonl` / `--format parquet` are priority. Build the parquet mode to make it easy to later tune the row group size, but for now we should auto-calculate a good medium value based on the size
- [ ] Support compressed output `--compress`, this should be responsive to the output format, e.g. gzip for jsonl and zstd for parquet

## Phase 2

- [ ] Allow ramping up parallelism with goroutines (`--workers`), workers fan out and run different parts of the timeline simultaneously, any areas of the simulation that don't support taking in a day and seed and providing idempotent output should be reworked to do so.

## Phase 3

- [ ] Implement `--messy` mode, there are a lot of aspects to this, and we'll have to play around with parameters to get it right, but we should be able to inject some realistic mess into the data -- badly formatted data, patches of inconsistent columns, negative amounts for absolute values like order_total, etc.

## Phase 4

- [ ] Expanded schema entities: payments, promotions, staff, loyalty program, reviews from social media sources, event logs, etc. We'll have to map out the exact set we want to add, but we should have all the patterns in place at this point that we can easily fan out an agent team to implement each new entity, and knock out this quickly after we handle the planning and schema design.

## Backlog

- TUI form (Charm/bubbletea)
  - At some point, we'll have enough options that it will be easier to launch a TUI and have the options laid out for you, so you can tune a run to your exact needs without editing the command line args directly
