---
id: T-2476
title: "MEDIAN, TRIMMEAN, SKEW, STDEV and pass-through last digits on the T-2256 gen-m9 sweep (29 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
MEDIAN, TRIMMEAN, SKEW, STDEV/STDEVP and the pass-through cells below give Excel's last digits on the T-2256 gen-m9 sweep,
from a fitted evaluation order, or the task records the candidates tried (decision 0056).

## Context
Gap triage of the T-2256 gen-m9 sweep. These 29 cells are within half a unit of the 15th significant digit (not visibly
off, so no campaign key) and no open accuracy task covers their outer function. MEDIAN (12): the mean of the two middle
values in some order, e.g. `MEDIAN(Data!C11:C11,Data!C8:C12)` Excel 3.4999499999999997, ours 3.49995 (a + (b - a)/2 or
a/2 + b/2 against (a + b)/2); TRIMMEAN (7): the sum of the kept values and the division; SKEW (3), STDEV/STDEVP (2),
PERMUTATIONA (2: `PERMUTATIONA(7,252)` and `(123456789.123,2)`, the power), MINA, NORMINV and TREND (1 each). Five cells
pass an inner function's last digit through (FDIST in MEDIAN, DDB in STDEVP, PRICE in MINA, IRR in NORMINV, IMARGUMENT in
TREND: T-2221, T-2033, T-2207, T-2131, T-2044); if the inner function's own task
fixes them first, close them here. Fit in the fit lab (tools, `rustsheet-fitlab`) on these cells plus the passing ones of
the same functions in eval/ (a candidate must not break a passing cell).
Code: functions/statistical/ (median in sorted_data.rs or count_and_average.rs; trimmean.rs; standard_dev.rs; permut.rs).
MEDIAN moved to T-3017 (gap triage 2026-10-28): `a + (b - a)/2` over the two middle values fits all 50 even-count
gen-m* MEDIAN cells over plain ranges (statistics.md "MEDIAN of an even count is a + (b - a)/2"). That covers 10 of
this list's MEDIAN cells, and T-3017 owns them; A264 and A168 pass an inner function's digits through and stay here.
More TRIMMEAN evidence (same triage, corpus-bench, Excel-cached; no task named it). In corpus-bench/50442/1_50442_input.xlsx
`TRIMMEAN(IF(Table1[MAX GAIN]>=4,Table1[POINTS G/L]),10%)` is 4.779374999999999 in Excel and 4.779375 in ours. All 7 cells
are 1 to 2 ulp off, and the case is a whole-case F line of eval/baseline/corpus-bench.txt:
- corpus-bench/50442/1_50442_input.xlsx (7): RESULTS 1!Q3, T3, Q4, T4, Q5, D52, D53

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 29 known-failure lines in eval/baseline/gen-m9.txt, 0 visibly off.
Examples:
- gen-m9/3b7990bd4e3307ba Calc!A149 `=TRIMMEAN(Data!C4:E9,0)`: Excel 6.018286249999999, ours 6.01828625
- gen-m9/73f053591a54bc43 Calc!A307 `=MEDIAN(Data!E9:E9,Data!D2:D11)`: Excel -0.09999999999999998, ours -0.1
- gen-m9/775d053de498b8c7 Calc!A162 `=MEDIAN(Data!C11:C11,Data!C8:C12)`: Excel 3.4999499999999997, ours 3.49995
- gen-m9/7fd0a98183c6babf Calc!A367 `=PERMUTATIONA(7,UNICODE("ü"))`: Excel 9.219472719891913e+212, ours 9.219472719891912e+212
- gen-m9/bbab6562e88b70e8 Calc!A170 `=MEDIAN(Data!D2:D10)`: Excel 0.6524999999999999, ours 0.6525
Cells:
- gen-m9/0bbab6206b5c9a07.case.json (1): Calc!A13
- gen-m9/139684e68fb2fc3b.case.json (1): Calc!A165
- gen-m9/201d67f1d59436ed.case.json (1): Calc!A314
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A87
- gen-m9/38e4fc309fe199ea.case.json (2): Calc!A63, A319
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A149
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A35
- gen-m9/5389dd4d5b5739d2.case.json (2): Calc!A31, A365
- gen-m9/6e18550e9fc9c1b9.case.json (2): Calc!A220, A336
- gen-m9/73f053591a54bc43.case.json (2): Calc!A239, A307
- gen-m9/775d053de498b8c7.case.json (2): Calc!A162, A286
- gen-m9/7fd0a98183c6babf.case.json (1): Calc!A367
- gen-m9/80586b859b73c87d.case.json (1): Calc!A264
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A114
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A117
- gen-m9/928d08e3e1eecb5f.case.json (1): Calc!A31
- gen-m9/9735c5ab72fa5ecf.case.json (1): Calc!A180
- gen-m9/a5342a7e24fd803e.case.json (1): Calc!A174
- gen-m9/bbab6562e88b70e8.case.json (1): Calc!A170
- gen-m9/c4635963fc926284.case.json (1): Calc!A174
- gen-m9/cb61169f4c033d4b.case.json (1): Calc!A168
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A30
- gen-m9/d910f000b60e2eac.case.json (1): Calc!A101
- gen-m9/eedd629f3d69fdda.case.json (1): Calc!A189

Calipers evidence (gap triage 2026-10-25, desktop goldens, ungated suite): `calipers/pv_goldens/pv23_049` WACC!E23 and
Public Comps!F17, F33, and `pv23_050` WACC!E23, `MEDIAN(E16:E21)` over {522.4, 118.75, 0, 0, 43.90399999999999,
12.302999999999999}: Excel 28.103499999999997, ours 28.103499999999993. Of the readings above, only lo + (hi - lo)/2 with
lo the smaller middle value (12.302999999999999 + (43.90399999999999 - 12.302999999999999)/2) gives Excel's value; (a + b)/2,
a/2 + b/2 and hi + (lo - hi)/2 give ours, in both single and double rounding. Fit it on the gen-m9 cells too.

## Acceptance criteria
- [ ] Last digits (gen-m9, T-2256): these 29 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Or the Outcome lists each cell still off with the candidates tried (decision 0056).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Visibly-off cells (other tasks).

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
