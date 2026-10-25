---
id: T-2147
title: "TAN near 1.2E+8 is 1 ulp off the pi64 reduction model (TAN(123456789.123))"
milestone: M5
lane: functions-math
status: done
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
- [x] The 1 gen-m6 known-failure line above disappears from eval/baseline/gen-m6.txt (`make eval-baseline`), or each remaining one is named in the Outcome with its cause.
- [x] These 4 known-failure lines (3 from T-2114, 1 from T-2252) disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] No m9-trig-reduction or trig-huge-argument cell regresses; T-2172's TAN cells pass or are listed residuals.
- [x] `make check` green; reviewers approve.

## Out of scope
- SIN/COS changes beyond what the sweep shows.

## Outcome
Excel's TAN is the pi64-reduced tangent rounded twice, RN53(RN64(tan)) (an x87 tangent stored to a double); `excel_tan` now
rounds that way (COT, 1/TAN, inherits it). Evidence: all 311 earlier constant-argument TAN cells, and the new
probes-numeric/t2147-tan-double-rounding (150 arguments chosen at 64-bit near-ties, excel-graph, two identical fills): 129 match,
against 47 for the old single rounding. Guard-bit variants (65..72 bits) do no better beyond noise (numbers.md "TAN misses this
model by 1 ulp"). The 21 residual cells of that probe (probes-numeric is not gated) are near-ties where Excel's own tangent
carries up to about half a 64-bit ulp of error, so its bit is not predictable from the exact value (as decision 0112 for LN).
Eval delta: gen-m5 known failures 9 -> 5 (the 4 lines above), gen-m6 2 -> 1 (the 1 line above); probes-numeric
trig-odd-even-sumxmy2-sweep 778 -> 782 passing; m9-trig-reduction 666/666 and probes/trig-huge-argument 86/86 unchanged; no regressions.
