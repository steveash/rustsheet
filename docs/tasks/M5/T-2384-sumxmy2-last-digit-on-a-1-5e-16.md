---
id: T-2384
title: "SUMXMY2 last digit on a 1.5E+16 sum: fit Excel's accumulation (SUMXMY2 is 1 ulp high)"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2172]
defer: phase-1b
---

## Goal
SUMXMY2 (and SUMX2MY2/SUMX2PY2 if the sweep shows the same accumulation) returns Excel's last digit on large sums.

## Context
Gap triage of gen-m5 round 2 (T-2252, excel-graph oracle, 2026-10-22). Needs an oracle sweep: T-2172 (its SUMXMY2 item, added by this triage). `SUMXMY2(C10:E12,C6:E8)` pairs (-1, 123456789.123),
(7, 1E-15) and (-1, 0.3); Excel gives 1.5241579027474522E+16, 1 ulp above the correctly rounded sum ...452E+16, which ours and the
left-to-right double sums of `(x-y)^2`, `x^2-2xy+y^2` and `x^2+y^2-2xy` all give (`SUMX2-2SUMXY+SUMY2` gives ...518). The cell is
not visibly off (2 units in the 17th digit), so the task is a last-digit tail, not in the campaign. Observation: numbers.md
"SUMXMY2 is 1 ulp above the correctly rounded sum on a 1.5E+16 result" (added by this triage). Candidates to test on the sweep:
an 80-bit accumulator, pairwise or reversed summation, squaring through the `^` kernel, a 15-digit cut of each term. Code:
`fn_sumxmy2` and its siblings in functions/math_and_trigonometry/.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/bf8903e05ec369ca Calc!A248 | `=SUMXMY2(Data!C10:E12,Data!C6:E8)` | 1.5241579027474522e+16 | 1.524157902747452e+16 |

## Acceptance criteria
- [ ] Every SUMX2MY2/SUMX2PY2/SUMXMY2 cell of the T-2172 sweep passes, or the remaining ones are listed residuals in numbers.md.
- [ ] This 1 known-failure line disappears from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pairing, blanks and error order (coercion.md, T-0961, done).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
