---
id: T-0142
title: "Oracle fill: a numeric web/desktop difference below 1e-9 keeps the desktop value (decision 0030)"
milestone: M1
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Where a sidecar-backed workbook has a desktop-Excel cached number, the cell is scored against the
desktop value, as decision 0030 rule 1 requires, even when the web differs only in the last digits.

## Context
A sidecar replaces a workbook's cached values entirely (`prepare_xlsx` in
crates/rustsheet-eval/src/runner.rs). `rustsheet-oracle fill`
(tools/src/rustsheet_tools/oracle/cli.py, the 1e-9 conflict check) drops cells whose web value
differs by more than 1e-9 relative, but writes the web value when the difference is smaller. So
last-digit web/desktop differences (poi, any `oracle_dir` suite) are scored against the web.

## Acceptance criteria
- [ ] When the web value of a number differs at all from a value cached by a Microsoft Excel
      application with AppVersion 16.x (decision 0030 rule 1/2; older builds and placeholder
      caches such as XlsxWriter never replace a web golden), the fill writes the desktop cached value into the sidecar (with a per-cell
      marker or a printed `DESKTOP` line) instead of the web value; differences above 1e-9 stay
      pending as before (decision 0019).
- [ ] Unit test in tools/ for the three cases (equal, below 1e-9, above 1e-9) and for a non-16.x producer.
- [ ] Re-filling the poi sidecars changes no scored cell from pass to fail (or the PR shows each
      changed cell with its desktop evidence).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`_drop_desktop_conflicts` (tools/src/rustsheet_tools/oracle/cli.py) now writes the cached desktop
number (and prints a `DESKTOP` line) when the workbook was saved by Microsoft Excel AppVersion 16.x
and the web number differs at all but within 1e-9; larger differences stay pending; pre-16 builds
keep the web value. Unit tests cover equal / below 1e-9 / above 1e-9 / non-16.x. Re-filling the poi
suite with `--force` changed only `produced_at` stamps and two volatile cells (46299 -> 46300,
a random value), no last-digit cell, so no sidecar changed and no scored cell moved. Eval delta: none.
