# Rowing Machine TODO

Phase numbers are stable IDs, not order; `**Dependencies**:` lines drive sequencing. `docs/architecture.md` shows the build lanes and which Phases can run as parallel sessions.

## Phase 13: Go retirement and first Rust release

**Dependencies**: 1, 2, 3, 14, 15
**Requirements**: R003, dev-R001, dev-R008, dev-R009, dev-R012, dev-R021, dev-R025, dev-R026

### Go retirement

- [ ] Confirm the parity test passes, then delete `go-reference/`, `tests/support/go_personas.go`, the `parity-capture` task, and the Go ignore rules
- [ ] Keep the parity fixture and test as a regression baseline, rewording `tests/fixtures/README.md` so it no longer offers a capture command
- [ ] Retire `dev-R021`, and drop Go references from `AGENTS.md`, `docs/architecture.md`, and the README note

### Repository provisioning

- [ ] Run `mise run ci-audit:pinact` to refresh action pins and `mise run ci-audit`
- [x] Run `mise run repo:settings --homebrew`, `mise run repo:labels`, and `mise run repo:environments`
- [ ] Open a throwaway PR with a deliberate lint failure and confirm the annotation lands on the diff
- [ ] #user Make the repository public; release archives, `cargo binstall`, Homebrew, and git-cliff's GitHub metadata all need it
- [ ] #user Confirm CONTRIBUTING and SECURITY resolve from the owner's `.github` repository
- [ ] #user Push `main` and run `mise run repo:rulesets` once CI reports on it

### Release

- [ ] Run `mise run release:rehearse`, resolve what it reports, and delete `docs/bootstrap.md`
- [ ] #user Create the Homebrew tap token secret, cut the release with `mise run release`, then run `mise run release:verify`
- [ ] #user Publish the first crate with `mise run release:bootstrap-crate`
- [ ] #user Add a crates.io trusted publisher for `release-build.yml` in the `release` environment, then set `CRATES_IO_PUBLISHING=true`
