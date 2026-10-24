---
id: T-2547
title: "BESSELJ and BESSELY above x = 100: #NUM! for huge x and the 15-digit misses at 1E+5 and 1.2E+8 (gen-m9, 7 cells)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: M
tier: opus
depends_on: [T-2140]
campaign: visible
---

## Goal
BESSELJ and BESSELY above x = 100 are within half a unit of Excel's 15th digit, and `#NUM!` where Excel's are for huge x.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep (docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)" "BESSEL"). Visibly off: `BESSELJ(1E+15,0)` and `BESSELJ(1E+15,5)` and `BESSELY(214285740740749.06,2)` are `#NUM!` (ours 5.29E-09, 2.43E-08, -2.30E-08); x = 123456789.123 computes but differs in the 12th digit (`BESSELJ(..,1)` 4.9869177786549744E-05, ours ...6651605E-05; `BESSELY(..,3)` 5.1668995876750464E-05, ours ...665215E-05), which suggests Excel's own SIN/COS (x87 reduction, near its 2^27 limit) inside the asymptotic form; `BESSELJ(1E+5,100000)` 0.009636944011336267 (ours ...629); `BESSELY(1000,47)` (the order `MEDIAN(..)` 47.529205) 0.006930090371433304 (ours ...299). The model of T-0908 (`asymptotic` in functions/engineering/transcendental/bessel_excel.rs) is evidenced for 8 <= x <= 100 only. Needs an oracle sweep: T-2140 asks Excel for x in {1E3, 1E6, 134217728, 1E10} at orders 0, 1, 3; extend that probe with x = 2^27-1, 1E+14, 2E+14, 1E+15 and orders 47 and 100000 at x = 1E+5 and 1000 before fitting. The last-ulp cells of the same model are T-2548.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 7 known-failure lines in eval/baseline/gen-m9.txt, 7 formulas, 7 visibly off.
Examples:
- gen-m9/2b0324e2de2e7b74 Calc!A295 `=BESSELJ(1E+5,100000)`: Excel 0.009636944011336267, ours 0.00963694401133629
- gen-m9/2f0d384b8ed0d392 Calc!A151 `=BESSELY(Data!A$4,3)`: Excel 5.1668995876750464e-05, ours 5.166899587665215e-05
- gen-m9/2f0d384b8ed0d392 Calc!A245 `=BESSELY((AVERAGE(Data!A2:A11,Data!C5,42)/(Data!C12/0.3)),2)`: Excel #NUM!, ours -2.302897880921332e-08
- gen-m9/408aa11aa923a4d9 Calc!A345 `=BESSELJ(MAX(Data!$B$11,Data!C3,Data!A6:A12),1)`: Excel 4.9869177786549744e-05, ours 4.9869177786651605e-05
- gen-m9/76b07b57099ccc8d Calc!A38 `=BESSELJ(Data!A12,Data!B7)`: Excel #NUM!, ours 5.286816888676355e-09
- gen-m9/7820ebc42c7a56a6 Calc!A316 `=BESSELY(1000,MEDIAN((0.2-Data!B8),(100-Data!D$10)))`: Excel 0.006930090371433304, ours 0.006930090371433299
Cells:
- gen-m9/2b0324e2de2e7b74.case.json (1): Calc!A295
- gen-m9/2f0d384b8ed0d392.case.json (2): Calc!A151, A245
- gen-m9/408aa11aa923a4d9.case.json (1): Calc!A345
- gen-m9/76b07b57099ccc8d.case.json (1): Calc!A38
- gen-m9/7820ebc42c7a56a6.case.json (1): Calc!A316
- gen-m9/f8bc18a0e45a0e1a.case.json (1): Calc!A202

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `BESSELJ(1E+15,100)` is `#NUM!` (ours 5.286816888554611E-09).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 1 known-failure line in eval/baseline/gen-m9.txt, 1 formula.
Examples:
- gen-m9/caa1eed48b1883fe Calc!A399 `=GAUSS((MIN(Data!A11:A12,Data!D4:D7)/BESSELJ(1000000000000000.0,100)))`: Excel #NUM!, ours -0.5; inner `BESSELJ(1000000000000000.0,100)` Excel #NUM!, ours 5.286816888554611e-9
Cells:
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A399

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2436): this known-failure line disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] (6 of 7; `BESSELJ(1E+5,100000)`, 2b0324e2de2e7b74 A295, Miller's recurrence, moved to T-2548) Visible parity (gen-m9, T-2258): these 7 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] engineering.md records Excel's x limit and the kernel change with the sweep cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
Probed the cut (probes/t2547-bessel-phase-cut, 88 cells, excel-graph 2026-10-24): BESSELJ and BESSELY are `#NUM!` once the
asymptotic phase (`x - 0.785398164` for order 0, `x - 2.356194491` for order 1) passes 134217727, the cut IMPOWER's angle
already had (now one constant, `EXCEL_INNER_TRIG_LIMIT` in trig.rs), below SIN's own 2^27; higher orders recur from orders 0
and 1, so they are `#NUM!` with order 0. The asymptotic form now takes Excel's own sine and cosine (`excel_sin_cos`),
which makes every cell of bessel-large-x and the new probe bit-exact (x = 1e3, 1e6, 1e5 orders 0-6, 2^27 order 1).
That also completes T-2563 (bessel-large-x, now all 24 pass) and fixes 5 of T-2548's last-ulp cells.
Two Mac-desktop cached cells of ironcalc-calc/ENGINEERING/BESSEL.xlsx (All!D18, F18, x = 1e5, order 5) differ from the
web's value, which the engine now gives bit for bit: recorded as known failures under decision 0109, desktop question
item 15 of T-1075. `BESSELJ(1E+5,100000)` (Miller's recurrence, n = x) is not fitted: moved to T-2548.
Eval delta: gen-m9 known failures 6753 -> 6732 (21 fixed), probes 2775 -> 2745 (30 fixed, new case all pass),
ironcalc-calc 736 -> 738 (the two decision-0109 cells); visibly-off cells 7341 (lock 8632).
