---
id: T-2824
title: "STDEV.S, VAR.S, SUMSQ, AVERAGEA and MAXA over a lifted OFFSET (T-2065 probes)"
milestone: M6
lane: engine
status: todo
priority: P0
size: S
tier: opus
depends_on: [T-2816]
campaign: semantics
---

## Goal
STDEV.S, VAR.S, SUMSQ, AVERAGEA and MAXA over an array of references (a lifted OFFSET) give Excel's values, so the
owned M6 exit cells pass.

## Context
- Decision 0188 (owner's 2026-10-31 rulings, R1(c)): this task owns probe-row or text cells of its milestone's exit,
  which Phase 1 still counts in full, so it is in the semantics stage (priority stays P0).
T-2065 (#1944) probed these on excel-graph (2026-10-26) and appended the values to T-2063, which was already done, so
the cells had no open owner (`make exit-owners MILESTONE=M6`, steward run 2026-10-26 18:47 UTC). Excel, with
single-cell elements: `STDEV.S(OFFSET(A1,{0;1},0))` 70.0035713374682, `VAR.S(..)` 4900.5, `SUMSQ(..)` 10001;
`AVERAGEA(OFFSET(A4,{0;1;2},0))` and `MAXA(..)` over "5", blank, TRUE are 0. Whether each element counts by its
top-left cell or as a whole range is not yet separated: T-2816 probes that, hence the dependency. Rules:
docs/specs/SPEC-0014-reference-array-results.md, docs/excel-semantics/lookup.md, T-2063's "T-2065 findings". Code:
the aggregate readers T-2063 extended (functions/statistical/, functions/math_and_trigonometry/).

Owned M6 exit cells (5; known failures in eval/baseline/probes.txt):
- probes/t2065-lift-stdevs Sheet1!E1
- probes/t2065-lift-vars Sheet1!E1
- probes/t2065-lift-sumsq Sheet1!E1
- probes/t2065-lift-averagea Sheet1!E1
- probes/t2065-lift-maxa Sheet1!E1

Same rule through INDIRECT (gap triage 2026-10-31c: no open task named these since T-2065 closed; statistics.md says
"The one-list functions read through the references"): probes/t2577-indirect-ref-array Calc!A157
`STDEV(INDIRECT("B1",{1;1}))` is 0 in Excel (B1 = 5 twice), ours `#VALUE!`; T157 and U157 are `ROWS(..)` and
`COLUMNS(..)` of it (Excel 1). They are M8 probe cells, not M6 exit cells, and follow the same aggregate reader.
- probes/t2577-indirect-ref-array.case.json (3): Calc!A157, T157, U157

## Acceptance criteria
- [ ] The three probes/t2577-indirect-ref-array cells above leave eval/baseline/probes.txt too.
- [ ] The five cells, and T-2816's probe cells, leave eval/baseline/probes.txt (`make eval-baseline`), with no
      regressions.
- [ ] `make exit-owners MILESTONE=M6` no longer lists them; lookup.md says the engine implements the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- COUNTIFS, AVERAGEIF and MAXIFS: T-2823.
