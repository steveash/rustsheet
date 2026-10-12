---
id: T-0411
title: "xlsx export writes DAYS, GAUSS, IMSECH, GAMMA.DIST and FORECAST.LINEAR without _xlfn."
milestone: M11
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A workbook exported by RustSheet stores every newer function with its `_xlfn.` prefix, so Excel does not read DAYS, GAUSS, IMSECH, GAMMA.DIST or FORECAST.LINEAR as `#NAME?`.

## Context
Found in T-0052. XlsxWriter (3.2.9) prefixes these five names (`_xlfn.DAYS`, `_xlfn.GAUSS`,
`_xlfn.IMSECH`, `_xlfn.GAMMA.DIST`, `_xlfn.FORECAST.LINEAR`). The engine implements all five
(`Function::Days`, `Gauss`, `Imsech`, `GammaDist`, `ForecastLinear`), but
`Function::to_xlsx_string` (crates/rustsheet-engine/src/functions/xlsx_names.rs) has no `_xlfn.` arm for
any of them, so the exporter writes them bare. A bare newer name read from a file is `#NAME?` in
Excel (docs/excel-semantics/xlsx-storage.md, "Newer functions need `_xlfn.`").
The case builder lists these names in `_BUILDER_ONLY_PREFIXES` (tools/src/rustsheet_tools/spec2xlsx.py)
until the exporter covers them. Other newer functions may be missing from the exporter too: compare
its list with the catalog.

## Acceptance criteria
- [x] `to_xlsx_string` returns the `_xlfn.` form for the five functions; a unit test covers them.
- [x] Remove the five names from `_BUILDER_ONLY_PREFIXES`, and check that no committed spec's built
  bytes change.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`Function::to_xlsx_string` now stores DAYS, GAUSS, IMSECH, GAMMA.DIST and FORECAST.LINEAR with `_xlfn.`; unit test added; the five names left `_BUILDER_ONLY_PREFIXES` (the builder reads the engine list). Eval: no change.
The comparison of the whole exporter list with the catalog (task Context) is filed as T-0412. Landed as a port of #330 onto `main` after T-1425 moved the list to `functions/xlsx_names.rs`.
