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
defer: phase-1b
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

gen-m6 triage (T-2180, excel-graph oracle, 2026-10-21): One more cell, TAN(123456789.123) inside CHOOSE, 1 ulp off as in the cells above.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 1 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/b62dd1ebd266b71c Calc!A39 | `=CHOOSE(Data!B7,TAN(Data!D4),3.14159,(SUM(Data!C6:C12)*(Data!$A9/Data!A$1)))` | 56.41575748007382 | 56.415757480073815 |

gen-m5 round-2 triage (T-2252, excel-graph oracle, 2026-10-22): one more cell, TAN(123456789.123) passed through a 1x1 MDETERM
(the innermost diverging call is the TAN; `MDETERM` of a scalar returns it unchanged).

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/02d59ba3e2c52190 Calc!A120 | `=MDETERM(TAN(Data!C1))` | 56.41575748007382 | 56.415757480073815 |

## Acceptance criteria
- [ ] The 1 gen-m6 known-failure line above disappears from eval/baseline/gen-m6.txt (`make eval-baseline`), or each remaining one is named in the Outcome with its cause.
- [ ] These 4 known-failure lines (3 from T-2114, 1 from T-2252) disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] No m9-trig-reduction or trig-huge-argument cell regresses; T-2172's TAN cells pass or are listed residuals.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SIN/COS changes beyond what the sweep shows.

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
