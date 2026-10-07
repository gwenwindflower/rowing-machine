---
name: Themes set scenario parameters and catalogs
date: 2026-10-06
requirements: [th-R011, th-R013, cl-R040]
status: accepted
---

# Themes set scenario parameters and catalogs

## Context

`th-R011` promised that switching themes changes only names and labels, and the project non-goals said themes never change a scenario's numbers. That kept themes simple, but it forced every variation in shape into a new scenario. A travel scenario that covers both a sparse point-to-point airline and a dense transit line would otherwise need separate scenarios with mostly duplicated logic, and a running shoe shop needs shoe prices rather than the catalog's café prices.

## Decision

A scenario is the skeleton (entities, columns, relationships, logic), and a theme is the skin. Scenarios declare parameters with defaults and ranges, and catalogs of typed records; themes set parameters under `[params.<scenario>]` and supply catalogs. `--param name=value` overrides a parameter for one run. Scenario logic may branch on parameter values where behavior really differs, replacing the earlier idea of separate modes.

Spec edits: `th-R011` narrows to themes with the same catalogs and parameters; `th-R019`–`th-R022` add parameters, catalogs, and the no-real-brands rule for bundled themes; `th-R013` lists `sneakers` and `airline`; `cl-R040` gives travel its own default theme; `cl-R043` adds `--param`.

## Alternatives considered

### Modes as a separate concept

A scenario would offer named modes (air, rail, transit) that themes choose. Rejected: modes are just bundles of parameter values, and a separate concept would duplicate what parameters express while blocking fine-grained overrides.

### Keep numbers out of themes

Keep `th-R011` as written and add scenarios for each shape. Rejected for the duplication described above.

## Consequences

### Positives

- One scenario can cover several shapes of a business, and a demo can tune a dataset without forking its theme.

### Negatives

- Switching themes can now change row counts and values, so lessons that pin a dataset pin the theme as well as the seed; the same theme and seed still give the same bytes (`R001`).
- Themes carry more validation surface: catalog fields and parameter ranges are checked before simulation.
