---
id: T-0044
title: "Evidence report: classify Excel evidence per cell, not per case"
milestone: M0
lane: harness
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The "Excel evidence per function" report (T-0020) counts a cell only when that cell's
expected value came from Excel.

## Context
Review findings on T-0020 (reviewer-excel-fidelity, 2026-10-05). `RunReport::build` uses
`is_excel_evidence(&case.expected_source)` once per case, and the source is decided per case:
- `prepare_spec` (runner.rs) gives a spec case with an oracle sidecar the sidecar's `source.kind`.
  So hand-written cells that the sidecar does not cover would also count as Excel evidence. That does
  not happen today (all 134 sidecars cover their cases), but it would with a partial fill.
- A suite with `use_cached_values = true` and `require_excel_authored = false` labels non-Excel
  cached values `excel-cached (<app>)`. That would count LibreOffice values. ironcalc-calc and poi both
  set `require_excel_authored = true` today, so this is not reachable yet.

## Acceptance criteria
- [ ] Each scored cell records whether its expectation came from Excel (oracle sidecar value or
  Excel-authored cached value). The evidence report counts only those cells. Unit tests cover a
  partly covered sidecar and non-Excel cached values.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Counting only the branches a formula actually evaluates (the counts stay an upper bound).

## Outcome
`CellResult.from_excel` records per cell whether the expectation came from an Excel oracle sidecar
(only cells the sidecar supplies) or an Excel-authored cached value; the evidence report counts only
those cells. Unit tests: partly covering sidecar (runner) and hand/non-Excel cells (report). No eval delta.
