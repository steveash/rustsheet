---
id: T-2603
title: "XNPV of an all-empty multi-cell values range is #VALUE! (probes/xnpv-empty-values-cell C8)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`XNPV(1,A2:A3,B11:B12)` with A2:A3 empty is `#VALUE!`, as Excel (probes/xnpv-empty-values-cell C8), while an empty cell inside a values range that also holds numbers stays `#NUM!` (probes/xirr-xnpv-whole-column-nogaps A1).

## Context
Follow-up of T-2599, which made one empty values cell a 0 cash flow. C8 is a known failure (ours `#NUM!`) with no explanation yet: the refusal of an all-empty range seems to be a different check from the whole-column one. Probe first (excel-probe): an all-empty 2-cell values range with 1-cell and 2-cell dates, a values range of an empty cell and a number (`A2:A3` with A3 = 5), and dates that are all empty. Code: `get_array_of_numbers_xpnv` in functions/financial/mod.rs. Semantics: docs/excel-semantics/financial.md "XNPV reads one empty values cell as a single 0 cash flow".

Cell added by gap triage (2026-10-31, `make eval` on origin/main, excel-graph; `scripts/exit_owners.py --all-off`
listed it as unattributed, and no open task named it):
- gen-m9/58e3253bbd3a7a82 Calc!A189 `=EXPONDIST(1E+307,EXP(XNPV(100,Data!B8:C8,Data!C1:D1)),"x")`: Excel `#VALUE!`, ours
  `#NUM!`. Data!B8:C8 (the values) is blank, FALSE: no number at all, the same family as C8. Data!C1:D1 (the dates) is
  FALSE, 123456789.123. Ours: XNPV `#NUM!`, and EXPONDIST passes it before reading "x". Either Excel's XNPV is `#VALUE!`
  here (the all-non-number values rule of C8) or EXPONDIST reads its text cumulative "x" first: add
  `XNPV(100,Data!B8:C8,Data!C1:D1)` itself and `EXPONDIST(1,1/0,"x")` to the probe to tell them apart, and hand the
  EXPONDIST order to a functions-stat task if it is that.
Cells:
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A189

## Acceptance criteria
- [ ] probes/xnpv-empty-values-cell C8 passes, and the new probe cells pass or their causes are recorded in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
