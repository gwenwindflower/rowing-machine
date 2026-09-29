# Rowing Machine TODO

Phase numbers are stable IDs, not order; `**Dependencies**:` lines drive sequencing. `docs/architecture.md` shows the build lanes and which Phases can run as parallel sessions.

## Phase 10: SaaS product usage

**Dependencies**: 5
**Requirements**: sp-R007, sp-R008, sp-R030, sp-R031, sp-R032, sp-R033, sp-R034, dev-R022, R001, R004

Adds sessions and events, the highest-volume SaaS entities. Can run in parallel with Phase 11; both register entities in `src/scenario/saas/mod.rs`, so fold the second one with care.

### Sessions

- [x] Generate sessions per user on a work-week rhythm in the account's region, with holiday dips
- [x] Model onboarding decay to a personal rate and the pre-churn fade

### Events

- [ ] Generate events inside each session from the theme's feature catalog, varying adoption by tier and role
- [ ] Tie activation events to `users.activated_at`
- [ ] Test session and event bounds, engagement-to-churn correlation, and volume scaling with `--scale`

### Docs and performance

- [ ] Add usage entities and example engagement SQL to the SaaS docs, and benchmark the scenario at default scale

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
