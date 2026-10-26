---
id: T-1895
title: "AMORDEGRC below period 1, at a cost of 1E308 and past 2^20 periods match Excel (probes fin-amor-period-bound B17, -b B2, -c A1-A5)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: opus
campaign: semantics
depends_on: [T-1841]
---

## Goal
AMORDEGRC returns Excel's value for a period between 0 and 1, for a cost whose charges round past 1E293, and past
2^20 periods at charges of 1E12 and more.

## Context
Found by T-1841's probes; both are base values (the period walk is not involved), so T-1841 left them as known
failures:
- probes/fin-amor-period-bound B17 `AMORDEGRC(2400,39679,39813,300,0.5,0.15,1)` is 485 in Excel, ours 0. 485 is the
  value of period 2 (`AMORDEGRC(..,2,..)` and `..,2.5,..` are 485, B18; period 1 is 776, probes/fin-argument-modes
  H18; period 0 is the first period's charge). Main's walk runs `floor(period)` periods, none for 0.5, and
  returns 0. AMORLINC at 0.5 is a full period's charge in Excel and ours (A8, A9: 360, 2.4).
- probes/fin-amor-period-bound-b B2 `AMORDEGRC(Z7,39679,39813,300,1E5,0.000001,1)` with Z7 = 1E308 is
  1.94700443465919E302 in Excel (15 significant digits), ours 0: `amor_round` pre-rounds `x*1E13`, which overflows
  to infinity past about 1.8E295, so the first period's charge is infinite and every later one 0
  (functions/financial/amortization.rs). Excel's value suggests its rounding keeps 15 significant digits there.
- Past 2^20 periods T-1841's kernel computes the decay in closed form (financial.md "AMORDEGRC past 2^20 periods", a
  DEVIATION), which drifts about 1e-13 relative from a walk; probes/fin-amor-period-bound-c (rate 1E-7):
  - A1 `AMORDEGRC(1E20,39679,39813,300,1100000,1E-7,1)`: Excel 18989305437102, ours 18989305437101 (main's walk ...102).
  - A3 `AMORDEGRC(1E20,39679,39813,300,2400000,1E-7,1)`: Excel 13720292047582, ours ...583 (main ...582).
  - A4 `AMORDEGRC(3E19,39679,39813,300,1500000,1E-7,1)`: Excel 5154670166168, ours ...167 (main ...168).
  - A5 `AMORDEGRC(Z1,39679,39813,300,2000000,1E-7,1)`, Z1 = 1E300: Excel 1.51632679480347E293 (15 significant digits),
    ours 1.5163267948033552E293, main 1.5163267948034761E293. Excel's value lies strictly between the closed form's and
    the walk's, the strongest sign that Excel rounds each step.
  - A2 `AMORDEGRC(1E20,39679,39813,300,1700000,1E-7,1)` passes (Excel 16344246350030), where main's walk was ...031: Excel's
    per-period arithmetic is not exactly a double walk at that size. A fit of Excel's step (a 15-digit rounding of each
    charge or of what is left, say) may explain both A2 and A5 and could give an exact bounded form; otherwise walk
    further (a scalar cut-off kept apart from the lifting weight is a fallback) or document.
Probe more points first (periods 0.1, 0.9, 1.5, 3.5 at rate 0.15; costs 1E290 to 1E308 at periods 0, 1 and 2).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 6 cells, 6 numeric past half a unit in the 15th digit and 0 with a wrong type, error or text, blamed on AMORDEGRC 6. Every one is listed above.

From T-2485 (2026-10-24): AMORDEGRC with 0 < period < 1 is unexplained (financial.md "AMORLINC and AMORDEGRC check
order"): mostly the asset's own period-1 charge (probes/amordegrc-fractional-period A1-A11, A23), but
`AMORDEGRC(10000,39000,39400,500,0.3,0.3,0)` is 1693 where its period 1 is 2284 (A24, A26; 1693 is the previous cell's
value), period 0.0001 is 0 (amor-argument-check-order A862, A863) and the 98882 asset at 0.5625 is 0 in a probe (A864) but 1
in gen-m9. One untested hypothesis is a value left over from an earlier evaluation: first probe each formula alone in a
fresh workbook (one formula per case) to test for calculation-order dependence; if it holds, decide (decision record)
between a DEVIATION and modelling it. These cells go with this task:
probes/amor-argument-check-order A846-A851 and probes/amordegrc-fractional-period's failing lines, and
gen-m9/d912671baa1364aa Calc!A76 `=AMORDEGRC(98882,37287,38647,SUMPRODUCT((1e-15*Data!A10)),TIMEVALUE("1:30 PM"),0.1,1)`
(Excel 1, ours 0; period 0.5625).
Cells:
- gen-m9/d912671baa1364aa.case.json (1): Calc!A76

## Acceptance criteria
- [ ] Visible parity: the 6 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 6 lines in eval/baseline/probes.txt disappear (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] fin-amor-period-bound B17, fin-amor-period-bound-b B2 and fin-amor-period-bound-c A1, A3-A5 pass (baseline lines removed), with no new failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The AMORDEGRC rate thresholds (T-1764).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (AMORDEGRC 0 or a different 13th-15th digit), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet when that target is reached move to a follow-up task filed without the key and named in the Outcome (decision 0083).

## Progress (in flight, 2026-10-26)
- Done on this branch, not yet checked or reviewed: `amor_round` skips its 13-decimal pre-round when `x * 1e13`
  overflows. fin-amor-period-bound-b B2 is now 1.9470044346591894E302, Excel 1.94700443465919E302: 15-digit
  (visible) parity, 1 ulp off at the bit level, so it stays a known failure (NumericUlp; no baseline change). Unit test,
  financial.md note and the lift test update are in; `make check` green at this head. Not reviewed, no PR yet.
- Untouched: B17 (period 0.5) and fin-amor-period-bound-c A1, A3-A5 (closed-form drift, still NumericClose).
