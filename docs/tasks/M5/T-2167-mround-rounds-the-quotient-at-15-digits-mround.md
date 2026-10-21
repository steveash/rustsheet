---
id: T-2167
title: "MROUND rounds the quotient up within 5E-15 of a half (MROUND(0.3,0.2) = 0.4)"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
campaign: visible
depends_on: [T-2201]
---

## Goal
MROUND rounds as Excel does where the binary quotient sits just below a half (`MROUND(0.3,0.2)` = 0.4).

## Context
- Evidence first (planner, 2026-10-20): the probe this task asks for is T-2201 (research), which fills `probes/t2201-mround-quotient-rounding`. Use that case; do not probe again. Its failing cells are this task's cells too.
- The rule (T-2201, VERIFIED on 213 excel-graph cells): docs/excel-semantics/numbers.md "MROUND rounds the quotient up within
  5E-15 of a half, and multiplies in binary". `q = |number/multiple|`; `k = floor(q)`, plus 1 when `q - k >= 0.5 - 5e-15`;
  result `k * |multiple|` with the sign of `number`. It is not a 15-digit cut of `q` (that misses `MROUND(1234.4999999999998,1)`
  = 1234). The 38 failing cells of probes/t2201-mround-quotient-rounding are this task's cells too.
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `fn_mround` (mathematical.rs) rounds `value/multiple` in binary: 0.3/0.2 = 1.4999999999999998 rounds to 1. Excel gives 0.4, which
fits a 15-digit cut of the quotient before rounding half away. Observation: docs/excel-semantics/numbers.md "MROUND rounds the
quotient as a 15-digit decimal" (added by this triage; T-2201 has since replaced it with the tolerance rule above, so
implement that rule, not a 15-digit cut).
- Gap triage 2026-10-21 (main 475eac98): T-2201 is done and its rule predicts all 213 probe cells, so no fitting is left:
  the task is now sonnet by the tier rubric (size S, evidence in eval/, mechanical criteria). All 39 cells are visibly off
  (for example `probes/t2201-mround-quotient-rounding M!C1` `=MROUND(A1,B1)`: Excel 0.4, ours 0.2; `M!C184`: Excel 1,
  ours 0; `M!C174`: Excel 0.4, ours 0.30000000000000004). The 38 probe lines in eval/baseline/probes.txt are
  `M!C1 C2 C3 C4 C7 C8 C11 C13 C15 C17 C19 C21 C23 C31 C33 C35 C37 C174 C184..C194 C201 C202 C204..C209` and `M!D174`.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/043ee562f32eb901 Calc!A199 | `=MROUND(0.3,Data!B6)` | 0.4 | 0.2 |

## Acceptance criteria
- [x] These 1 known-failure lines disappear from eval/baseline/gen-m5.txt, and the 38 `probes/t2201-mround-quotient-rounding`
      lines from eval/baseline/probes.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] Unit tests in mathematical.rs cover a quotient just inside and just outside `0.5 - 5E-15`, a negative number and
      `MROUND(0.5,0.2)` = 0.6000000000000001 (binary product), citing the probe cells.
- [x] A comment at the change cites docs/excel-semantics/numbers.md "MROUND rounds the quotient up within 5E-15 of a
      half, and multiplies in binary" (T-2201 already replaced the 15-digit observation there).
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: every cell this task owns is visibly off (a wrong rounding step). The acceptance criteria above are the
visible-parity stage.

## Out of scope
-

## Outcome
`fn_mround` now uses `mround_value` (ceiling_floor.rs): quotient fraction >= 0.5 - 5e-15 rounds up, binary product, +0 for zero. Eval: gen-m5 -1 known failure, probes -38; visible-off cells 1765 -> 1726. Unit test `mround_quotient_tolerance`.
