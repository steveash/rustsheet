---
id: T-0441
title: "Data tables: load <f t=dataTable> cells and compute TABLE() results (probes/data-tables)"
milestone: M11
lane: engine
status: blocked
priority: P3
size: M
tier: opus
depends_on: [T-0401]
---

## Goal
What-if data tables (one-variable column and row tables, two-variable tables, chained tables) import from
xlsx and calculate as Excel does, so cells that read their results see Excel's values, and `ISFORMULA` and
`FORMULATEXT` of a result cell match (`{=TABLE(,A1)}`).

## Context
Gap triage 2026-10-19: probes/data-tables (excel-graph, 2026-10-16, T-0428) fails 15 cells and no open task
covers them (T-0428 is done; T-0429 scored only the iteration probes):
- Result cells S!B4, D8, B11, B15: Excel 12, 12, 110, 104; ours empty (the importer drops data tables).
- Readers S!H1 `SUM(B4:B6)` 66, H2 `B5` 22, H3 `INDEX(B4:B6,3)` 32, H4 `SUM(D8:F8)` 66, H5 `E8` 22,
  H6 `SUM(B11:D12)` 1290, H7 `C12` 220, H9 `SUM(B15:B16)` 228, H10 `COUNT(B4:B6)` 3: ours 0.
- H11 `ISFORMULA(B5)` TRUE (ours FALSE), H12 `FORMULATEXT(B4)` "{=TABLE(,A1)}" (ours #N/A).
Rules: docs/excel-semantics/recalc.md (data tables are recalculated; sweeps run in place, in sheet order;
FORMULATEXT shows `{=TABLE(,A1)}`). docs/milestones.md: TABLE() parity is M11 scope and "the
implementation needs the M3 graph". spec2xlsx already writes `<f t="dataTable">` from a case's
`data_tables` (T-0428); `CaseSpec`/`SheetSpec` carry it and the eval adapter ignores it.
A data table re-evaluates its formula row/column once per input value with the input cell substituted,
which needs the dependency graph's recalculation of a substituted cell (SPEC-0002, T-0401), so this is
blocked on it. Directive 5: the number of substitutions is rows x columns of the table; it needs a
work bound like the other walkers.

## Acceptance criteria
- [ ] The 15 probes/data-tables lines leave eval/baseline/probes.txt (`make eval-baseline`) with no new F
      lines; a round-trip test keeps the `dataTable` formula on export.
- [ ] recalc.md's data-table entry gains an "Ours" line; any work cap is a DEVIATION entry if Excel has none.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Iterative calculation and circular references (T-0401's implementation tasks).

## Blocked because
Needs the M3 dependency graph and its recalculation of a substituted input (SPEC-0002, T-0401); M3 tasks
stay blocked until Phase 1 exits (docs/milestones.md). Unblocked when T-0401 is done and its implementation
tasks provide a way to recalculate a formula with one cell's value substituted.
