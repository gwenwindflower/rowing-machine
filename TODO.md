# Rowing Machine TODO

Phase numbers are stable IDs, not order; `**Dependencies**:` lines drive sequencing. `docs/architecture.md` shows the build lanes and which Phases can run as parallel sessions.

## Phase 5: SaaS accounts and revenue

**Dependencies**: 3
**Requirements**: sp-R001, sp-R002, sp-R003, sp-R004, sp-R005, sp-R006, sp-R010, sp-R011, sp-R012, sp-R013, sp-R014, sp-R015, sp-R016, sp-R020, sp-R021, sp-R022, sp-R023, sp-R024, th-R006, th-R010, th-R013, cl-R041, op-R002, sm-R034, dev-R017, dev-R022, R001, R002, R004

Stands up the `saas` scenario with accounts, users, plans, subscriptions, MRR movements, and invoices. Accounts arrive through a simple arrival stage with `direct` attribution; Phase 11 replaces that stage with the marketing funnel without changing the account lifecycle.

### Scenario scaffold

- [x] Register `saas` in the scenario registry and wire `--scenario`
- [x] Declare the SaaS name kinds and label sets (organizations, plans, features, campaigns, industries, roles, regions) and add generators for them to `plain`
- [x] Implement staged generation: an account arrival stage by day, then one lifecycle unit per account

### Account lifecycle

- [x] Generate accounts, users, and seat growth scaled by employee band
- [x] Generate trials, conversion driven by user activation, plan and interval choice, and subscriptions
- [x] Generate expansion, contraction, involuntary churn from unpaid invoices, voluntary churn by tenure and engagement, and reactivation

### Revenue ledger

- [ ] Derive MRR movements from subscription changes and classify each movement type
- [ ] Generate invoices that tile each subscription's active period, with late and unpaid payments
- [ ] Test the `sp-R010`–`sp-R016` invariants, MRR by date, and signup-cohort retention shape from the output files

### Docs

- [ ] Add the SaaS entities to `docs/output-schema.md` and write `docs/saas-model.md` with the lifecycle model and example MRR and cohort SQL

## Phase 10: SaaS product usage

**Dependencies**: 5
**Requirements**: sp-R007, sp-R008, sp-R030, sp-R031, sp-R032, sp-R033, sp-R034, dev-R022, R001, R004

Adds sessions and events, the highest-volume SaaS entities. Can run in parallel with Phase 11; both register entities in `src/scenario/saas/mod.rs`, so fold the second one with care.

### Sessions

- [ ] Generate sessions per user on a work-week rhythm in the account's region, with holiday dips
- [ ] Model onboarding decay to a personal rate and the pre-churn fade

### Events

- [ ] Generate events inside each session from the theme's feature catalog, varying adoption by tier and role
- [ ] Tie activation events to `users.activated_at`
- [ ] Test session and event bounds, engagement-to-churn correlation, and volume scaling with `--scale`

### Docs and performance

- [ ] Add usage entities and example engagement SQL to the SaaS docs, and benchmark the scenario at default scale

## Phase 11: SaaS marketing and funnel

**Dependencies**: 5
**Requirements**: gm-R001, gm-R002, gm-R003, gm-R004, gm-R010, gm-R011, gm-R012, gm-R013, gm-R014, gm-R020, gm-R021, gm-R022, gm-R023, gm-R040, sp-R001, sm-R034, dev-R022, R001, R004

Replaces Phase 5's direct arrival stage with campaigns, spend, touches, and leads that convert into accounts.

### Marketing

- [ ] Generate campaigns per channel with budgets and flights, and daily `ad_spend` with impressions, clicks, and spend
- [ ] Generate paid touches from clicks and organic, referral, and direct touches with steady growth
- [ ] Give visitors multi-touch paths so first-touch and last-touch attribution disagree

### Funnel

- [ ] Convert touches to leads by channel quality, and route leads to trials or demo requests by employee band
- [ ] Feed converted leads into the account lifecycle as its arrival stage, setting `acquisition_channel` and `first_touch_id`
- [ ] Test funnel monotonicity, lead-to-account tracing, and paid CAC per channel from the output files

### Docs

- [ ] Document the funnel model with example attribution and CAC SQL

## Phase 12: SaaS sales pipeline

**Dependencies**: 11
**Requirements**: gm-R005, gm-R006, gm-R007, gm-R008, gm-R030, gm-R031, gm-R032, gm-R033, gm-R034, gm-R035, gm-R041, gm-R042, dev-R022, R001, R004

### Sales team and opportunities

- [ ] Generate the rep roster by segment with hiring, departures, ramp, and annual cost
- [ ] Generate opportunities from demo leads with stage progression, band-driven cycle length and amount, and quarter-end close pressure
- [ ] Generate sales activities within each opportunity's open window

### Closing the loop

- [ ] Start the account's first paid subscription at each won opportunity's close, matching amount to ARR
- [ ] Test stage order, owner employment, activity windows, win rate, and blended CAC and payback bounds

### Docs

- [ ] Document the sales model with example pipeline, win-rate, and blended CAC SQL

## Phase 15: Parallel output throughput

**Dependencies**: 2
**Requirements**: sm-R035, sm-R036, sm-R033, op-R017, op-R018, op-R023, R001, dev-R016, dev-R017, dev-R020, dev-R031, dev-R032

Phase 2 wired a worker pool but gained nothing: at scale 100 (4.2M rows) one worker takes 10.5 s and ten workers 10.9 s, with user time about equal to wall time, so one core does nearly all the work. A profile puts most of the time in the serial `OutputSink::write` path, which validates each row, serializes every field to CSV strings (even for JSONL and Parquet) to build its primary key, clones those strings into an ever-growing key set, and then serializes the row again in the format writer. UUID hex formatting through `core::fmt` is the hottest leaf. The scheduler also stops the pool while it writes each batch, so generation and writing never overlap. Serial throughput fell from 717k to 457k rows per second between the Phase 9 baseline and the Phase 2 measurement. Owns `src/engine/` scheduling, `src/output/`, and the bench and profile tasks; it can run alongside the SaaS Phases, which only add scenario code.

### Profiling harness

- [ ] Add `mise run profile` that builds with symbols into a scratch target dir and records a profile (`samply` on macOS and Linux) of a given invocation
- [ ] Add the scale-100 run to `mise run bench` and record one-worker and all-core baselines in `docs/performance.md`
- [ ] Bisect the serial regression between the Phase 9 baseline and Phase 2, and note the cause in `docs/performance.md`

### Row path off the serial thread

- [ ] Move row validation and format serialization into the worker that generated the unit, handing the writer finished bytes per entity
- [ ] Replace string key sets with typed keys (UUIDs as `u128`) in a hash set, or prove key uniqueness by construction and check it in tests only, keeping `op-R018`
- [ ] Format UUIDs, timestamps, and integers into reusable buffers without `core::fmt`

### Overlapped ordered writing

- [ ] Stream finished units to writers through a bounded, ordered handoff so workers keep generating while earlier units are written
- [ ] Write each entity's file on its own thread, or show with the profile that one writer thread keeps up
- [ ] Keep Parquet and compressed JSONL byte-identical across worker counts, with encoding in parallel where the format allows it

### Proof

- [ ] Show `sm-R035` and `sm-R036` in `mise run bench`, keep every byte-identity test passing at 1 and all workers, and update `docs/performance.md`

## Phase 13: Go retirement and first Rust release

**Dependencies**: 1, 2, 3, 14, 15
**Requirements**: R003, dev-R001, dev-R008, dev-R009, dev-R012, dev-R021, dev-R025, dev-R026

### Go retirement

- [ ] Confirm the parity test passes, then delete `go-reference/` and the Go ignore rules
- [ ] Drop Go references from `docs/architecture.md` and keep the parity fixture as a regression baseline

### Repository provisioning

- [ ] Run `mise run ci-audit:pinact` to refresh action pins and `mise run ci-audit`
- [x] Run `mise run repo:settings --homebrew`, `mise run repo:labels`, and `mise run repo:environments`
- [ ] Open a throwaway PR with a deliberate lint failure and confirm the annotation lands on the diff
- [ ] #user Confirm CONTRIBUTING and SECURITY resolve from the owner's `.github` repository
- [ ] #user Push `main` and run `mise run repo:rulesets` once CI reports on it

### Release

- [ ] Run `mise run release:rehearse`, resolve what it reports, and delete `docs/bootstrap.md`
- [ ] #user Create the Homebrew tap token secret, cut the release with `mise run release`, then run `mise run release:verify`
- [ ] #user Publish the first crate with `mise run release:bootstrap-crate`
- [ ] #user Add a crates.io trusted publisher for `release-build.yml` in the `release` environment, then set `CRATES_IO_PUBLISHING=true`
