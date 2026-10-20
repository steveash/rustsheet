---
id: T-2167
title: "MROUND rounds the quotient at 15 digits (MROUND(0.3,0.2) = 0.4)"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
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
quotient as a 15-digit decimal" (added by this triage). Probe first (`excel-probe`): near-half quotients (0.3/0.2, 0.7/0.2,
2.5/1, 1.15/0.1, 1.05/0.1), negatives, and the product's last digit (`MROUND(0.5,0.2)`).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/043ee562f32eb901 Calc!A199 | `=MROUND(0.3,Data!B6)` | 0.4 | 0.2 |

## Acceptance criteria
- [ ] These 1 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] numbers.md entry updated with the probe results.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
