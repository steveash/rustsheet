---
id: T-2147
title: "TAN near 1.2E+8 is 1 ulp off the pi64 reduction model (TAN(123456789.123))"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2172]
---

## Goal
TAN returns Excel's doubles for arguments near 1.2E+8 (`TAN(123456789.123)` = 56.41575748007382).

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). Needs an oracle sweep: T-2172. `excel_tan` (trig.rs) is the pi64-reduction model that fits all 672 cells of m9-trig-reduction and
trig-huge-argument (numbers.md, T-1563), but it misses these by 1 ulp; `SIN(x)/COS(x)` gives the same miss. Find whether
Excel's tangent there is the quotient of separately rounded x87 FSIN/FCOS, an FPTAN result rounded twice, or a different reduction
for large n. Observation: numbers.md paragraph "TAN misses this model by 1 ulp at arguments near 1.2E+8" (added by this triage).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 3 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/c688ca172fe533dc Calc!A284 | `=TAN(((Data!A12*Data!B9)+((-1)^Data!B12)))` | 0.6173408433901317 | 0.6173408433901318 |
| gen-m5/d0419fa09d6a756a Calc!A311 | `=TAN(123456789.123)` | 56.41575748007382 | 56.415757480073815 |
| gen-m5/f0f205865a3a4328 Calc!A227 | `=TAN(SUMIF(Data!C5:E6,">=2"))` | 56.41575748007382 | 56.415757480073815 |

## Acceptance criteria
- [ ] These 3 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] No m9-trig-reduction or trig-huge-argument cell regresses; T-2172's TAN cells pass or are listed residuals.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SIN/COS changes beyond what the sweep shows.
