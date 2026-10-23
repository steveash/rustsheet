---
id: T-1538
title: "F-distribution quantile and cdf at huge df (1E7 to 1E10) differ from Excel in the 6th digit"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
F.INV, F.INV.RT, FINV, F.DIST and FDIST at a huge df (1E7 up to 1E10) match Excel's values to the last digits, e.g. `F.INV(0.5,1E10,1E10)` = 1 and `F.INV(0.5,10000000000,5)` = 1.1490395521130337.

## Context
T-1530 bounded the quantile (bisection on the `statrs` cdf, `f_quantile` in `functions/statistical/fisher.rs`) so it no longer hangs, but `statrs`' cdf is only good to about 1E-6 at huge df (same as Student-t, see T-0967). Failing cells: 16 of 58 in probes-numeric `t1530-f-df-cut` (e.g. A1 gives 1.149036767064711), and `F.INV(0.5,1E10,1E10)` gives 1.0000025. The pdf cells A9/A10 (`F.DIST(...,FALSE)` at df 1E10) are `#NUM!` for us, a type mismatch, not precision. Excel's `F.INV` and `F.INV.RT(1-p)` also differ at df 1E10 (A1 vs A3, A2 vs A4; statistics.md), so the two cannot share one quantile path. Needs a fit or an own incomplete-beta kernel; see docs/excel-semantics/statistics.md (F-distribution df limit).

gen-m9 sweep (gap triage 2026-10-23, T-2255). F.INV, FINV and F.INV.RT with a df of 1E6 or more (16 cells,
`F.INV(0.05,3,1E+8)` Excel 0.11728210553634685, ours 0.1172821026966143).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 16 known-failure lines in eval/baseline/gen-m9.txt, 16 formulas.
Examples:
- gen-m9/0a9226016e8f5f33 Calc!A358 `=BETAINV(1E-20,1E+10,0.5)`: Excel 0.9999999956419133, ours 0.9999999956419119
- gen-m9/85f48d5cdfb24555 Calc!A315 `=FINV(1E-20,10000000000.5,30)`: Excel 49.36450125047891, ours #NUM!
Cells:
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A358
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A131
- gen-m9/589f42e67f4b4b8d.case.json (2): Calc!A74, A319
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A28
- gen-m9/7c20351562a2d71d.case.json (2): Calc!A37, A254
- gen-m9/85f48d5cdfb24555.case.json (2): Calc!A214, A315
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A371
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A169
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A308
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A39
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A170
- gen-m9/f34f66fb7f81e673.case.json (2): Calc!A117, A209

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 16 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] `t1530-f-df-cut` cells A1-A14 and A57-A58 pass bit-exact or the residual is recorded as a DEVIATION.
- [ ] Cells at df 99999, 1E5 and 1E6 with extreme p (1E-12, 1-1E-12) are probed, to justify the bisection switch-over (`F_QUANTILE_BISECT_DF`).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): the gen-m9 cells listed under Context are visibly off (a wrong type, error or
text, or a number further than half a unit in the 15th significant digit from Excel's), so the task is in the campaign. Its
target is that every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states
it. Bit-level criteria still unmet then move to a follow-up task without the key.

## Resume notes (2026-10-23, opus run; session budget reached during research)
No engine change yet. Findings (docs/excel-semantics/statistics.md "F quantile at a huge df goes through Excel's
BETA.INV (T-1538)"; probes-numeric t1538-f-inv-huge-df and t1538-beta-dist-at-excel-inv, excel-graph):
- Our F quantile at df 1E10 is the true quantile (`F.INV.RT(0.5,1E10,5)` ours 1.1490395821369899, true
  1.14903958213698623). Excel's is not: its F.INV is Excel's BETA.INV mapped to F, and that BETA.INV stops short of
  its own (accurate) CDF's crossing by up to about 1E-16 in x. So the owned cells need Excel's BETA.INV search
  reproduced at a huge shape, not a better forward kernel. The F.DIST cdf cells (A7, A8, A11-A14) are a forward
  question: Excel's F.DIST there is the true value. T-2528 fixed the 7E-9 miss (the reflected continued fraction
  did not converge); what remains is about 10 ulp (A7 0.7764950711139107 vs Excel ...097, A8 ...177 vs ...169),
  the large-shape prefactor's precision (`large_prefactor` cancels a ln x + b ln y against ln B; TOMS 708 `bgrat`
  for a huge b is a candidate).
- F.INV(p,d1,d2) = 1/F.INV.RT(p,d2,d1) within 2 ulps everywhere probed, so one search per (p, smaller-df-first)
  pair serves both.
- Search clue: for x above about 1E-9, Excel's BETA.INV lands on a 2^-52/2^-53 grid and plain bisection of [0, 1]
  reproduces it (shape 2.5); below, it is off-grid and neither bisection, interpolation in the last bracket nor
  Newton from the mean reproduces it. Next: probe BETA.INV on a sweep of p at b = 5E9 (and 5E5, 5E7) to find
  where the grid behaviour stops, and test bisection in log x or a bracket that starts at the mean.
- Out of reach for a quick fix: the `F.INV(1E-200,1E10,6)` cells (Excel 1.30E-7, whose true CDF is about
  1E-9994283) are Excel's search failing outright.
