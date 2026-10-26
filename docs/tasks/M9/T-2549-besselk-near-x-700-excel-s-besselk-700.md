---
id: T-2549
title: "BESSELK near x = 700: Excel's BESSELK(700,3) is 4.69979514014056E-306, ours 4.6998714E-306 (gen-m9, 1 cell)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: [T-2550]
campaign: accuracy
---

## Goal
BESSELK near its underflow (x about 700) is within half a unit of Excel's 15th digit.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep (docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)" "BESSEL"). `BESSELK(700,3)` is 4.69979514014056E-306 in Excel and 4.699871435825504E-306 in ours (gen-m9/8cbfd5e99d2e82da A166; 1.6E-5 relative). With BESSELK(713,n) and (714,n) flushing to 0 in Excel (T-2421), Excel's K kernel seems to lose precision as e^-x nears the bottom of the double range. Needs an oracle sweep: one cell cannot fit a kernel, so T-2550 asks Excel for x = 500 to 713 at orders 0 to 5 first.
Code: functions/engineering/transcendental/bessel_k.rs.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas, 1 visibly off.
Examples:
- gen-m9/8cbfd5e99d2e82da Calc!A166 `=BESSELK(700,3)`: Excel 4.69979514014056e-306, ours 4.699871435825504e-306
Cells:
- gen-m9/8cbfd5e99d2e82da.case.json (1): Calc!A166

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2258): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Every T-2550 sweep cell (the 12 listed in the update below) matches Excel exactly, or the time-box (decision 0056) is recorded in engineering.md with the best candidate.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Update (T-2550)
The sweep is done: 12 cells differ (n = 2 to 5 at x = 700, 702, 705), not only `BESSELK(700,3)`. At 702 and 705 Excel's
K2 = K4 = K0 and K3 = K5 = K1. Rule to implement: Excel sets a subnormal recurrence product `2n/x*K(n)` to 0 (replayed bit-exactly on all 12 cells). See docs/excel-semantics/engineering.md "BESSELK near the bottom of the double range,
BESSELI overflow point (T-2550)" and probes-numeric/m9-besselk-underflow.

## Stage (decision 0114)
`campaign: accuracy`: a 50-digit reference shows Excel is close to correctly rounded for this family and the engine is not. Aim for a correctly rounded kernel, scored against an mpmath reference and the oracle; do not fit Excel's last bits. Cells left within 4 ulp after that go to a `defer: phase-1b` follow-up.

## Outcome
`bessel_k`'s upward recurrence now flushes a subnormal term `2j/x * K(j)` to 0 before adding it, the rule T-2550 replayed
bit-exactly. No fitting: it is a semantic flush, and K0/K1 were already bit-exact there. probes-numeric/m9-besselk-underflow
goes from 134 to 146 of 147 cells (all 12 BESSELK cells at x = 700, 702, 705; the last cell is the BESSELI cut, T-2613).
New unit test `besselk_recurrence_flushes_a_subnormal_term`. Eval delta: gen-m9 known failures 4548 -> 4547 (Calc!A166),
0 regressions in every gated suite.
