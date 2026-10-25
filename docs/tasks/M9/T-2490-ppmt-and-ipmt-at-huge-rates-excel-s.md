---
id: T-2490
title: "PPMT and IPMT at huge rates: Excel's 0, overflow and long-tail results (gen-m9, 31 cells)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
PPMT and IPMT return Excel's value at huge rates, where Excel's result is 0 (or another finite value) and ours a huge number, 0 or `#NUM!`.

## Context
Gap triage of the T-2257 gen-m9 financial sweep. At rates of 10 and above (and 1E+300) Excel's PPMT is usually exactly 0
while ours is huge: `PPMT(1E+10,12,278,130633)` is 0 (ours -1.3E15), `PPMT(42,10,303,8292,1,1)` 0 (ours -168.000000000008),
`PPMT(1E+300,10,193,369279,0,1)` 0 (ours `#NUM!`), `IPMT(1E+10,10,320,2)` is -20000000000 (ours 0), `IPMT(1E+10,8,226,439138,
43)` -4391380000000000 (ours 0), `IPMT(123456789.123,3,156,64486)` -7961234503385.777 (ours -16181728263929.855),
`IPMT(1E+300,3,2000,452017,905)` -4.52017E+305 (ours `#NUM!`), and `PPMT(10,11,304,44626,826,0)` -3.08E-301 (ours 0). This
looks like Excel computing PPMT as PMT - IPMT in doubles (cancelling to 0 when `(1+r)^n` swamps) and IPMT from the
balance; financial.md "PMT, IPMT and PPMT are the exact payment" (T-2130) moved ours to an exact form that does not cancel.
Fit Excel's formulation on these cells plus probes-numeric/m9-fin-pmt (no regression there) before changing annuity.rs;
if the cells are not enough to fix the formulation, file a research sweep task and mark this one blocked on it.
docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)".
Code: `fn_ppmt`, `fn_ipmt` in crates/rustsheet-engine/src/functions/financial/annuity.rs.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 31 known-failure lines in eval/baseline/gen-m9.txt, 31 visibly off.
Examples:
- gen-m9/31474c20fc1dafd6 Calc!A235 `=PPMT(42,10,303,8292,1,1)`: Excel 0.0, ours -168.00000000000819
- gen-m9/6376e27be7a05871 Calc!A32 `=PPMT(10,11,304,44626,826,0)`: Excel -3.0768304255991313e-301, ours 0.0
- gen-m9/67a4c03afd63f988 Calc!A15 `=IPMT(123456789.123,3,156,64486)`: Excel -7961234503385.777, ours -16181728263929.855
- gen-m9/762cd0e4ec9a7aa9 Calc!A292 `=IPMT(Data!$A9,13,255,24491)`: Excel -3023580222411.393, ours 0.0
- gen-m9/abb7d565af89b849 Calc!A112 `=IPMT(1E+300,3,2000,452017,905)`: Excel -4.5201700000000004e+305, ours #NUM!
- gen-m9/b6a02893a8d6b151 Calc!A280 `=IPMT(1E+10,8,226,439138,43)`: Excel -4391380000000000.0, ours 0.0
Cells:
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A327
- gen-m9/1dce4774f5c0e736.case.json (2): Calc!A259, A308
- gen-m9/30e20ba3302fea8b.case.json (1): Calc!A291
- gen-m9/31474c20fc1dafd6.case.json (1): Calc!A235
- gen-m9/3aa168259335be54.case.json (1): Calc!A340
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A109
- gen-m9/5422bdbf173fc233.case.json (1): Calc!A285
- gen-m9/5c6a23c4f6597f1c.case.json (1): Calc!A78
- gen-m9/6376e27be7a05871.case.json (2): Calc!A32, A179
- gen-m9/67a4c03afd63f988.case.json (3): Calc!A15, A278, A376
- gen-m9/762cd0e4ec9a7aa9.case.json (2): Calc!A201, A292
- gen-m9/81bcdc0bec0c8e6b.case.json (2): Calc!A234, A249
- gen-m9/89f0aee953a55a45.case.json (1): Calc!A124
- gen-m9/9a1fa5366bc50918.case.json (2): Calc!A31, A80
- gen-m9/a91bad4e84f49739.case.json (2): Calc!A109, A158
- gen-m9/abb7d565af89b849.case.json (1): Calc!A112
- gen-m9/b6a02893a8d6b151.case.json (2): Calc!A280, A348
- gen-m9/b72d952c5f52cb4d.case.json (1): Calc!A168
- gen-m9/b73c6cc96e093f03.case.json (1): Calc!A107
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A167
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A205
- gen-m9/c0681a8b150ce301.case.json (1): Calc!A81

T-2496 probe (probes/t2496-fin-subexpressions, each case's Data sheet copied as Dnn, every call and compound argument read through a TYPE/shape summary, excel-graph, 2026-10-25; probes/tbill-error-order-inner-calls for the T-bill cells): the innermost diverging sub-expression of 1 gen-m9 financial cell moved here from T-2496.

| gen-m9 cell | innermost sub-expression | Excel | ours |
|---|---|---|---|
| gen-m9/81bcdc0bec0c8e6b Calc!A251 | `PPMT((2*Data!B11),8,360,(-100),926)` (rate 2E+15; inside TBILLEQ) | 0 | 2E+17 |

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2257) (30 of 31; A158 moved to T-2666; the new probe cases add known-failure lines of their own, no existing case regressed): these 31 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] No PMT, IPMT or PPMT cell of probes-numeric/m9-fin-pmt or IPMT_PPMT.xlsx gets visibly off (re-check with `make eval-show`).
- [x] Visible parity (gen-m9, T-2496): the known-failure line of the T-2496 probe block above disappears from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] `make check` green; reviewers approve.

## Out of scope
- The long-term tails within a few digits (T-2242).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
Probed (probes/fin-ipmt-ppmt-huge-rate, 1194 cells, and probes/fin-ppmt-discount-underflow, 81 cells; excel-graph,
2026-10-25) and fitted: IPMT is `-rate*B` on the balance `B = (pv+fv)*Y - fv`, in doubles on the rounded `Y` where it
cancels (`|B| < |fv|`), so pv-0 cells carry fv's rounding (`IPMT(0.5,2,124,0,315)` is 0); past `(1+rate)^nper`'s range IPMT
keeps that form (about `-pv*rate`) and PPMT the discounted form with a subnormal discount read as 0; neither needs PMT's
value, and IPMT is 0 in period 1 of type 1 even where PMT is `#NUM!`. financial.md "IPMT cancels on fv".
Eval: gen-m9 known failures 5340 -> 5305 (35 fixed: 30 of this task's 31 lines, the T-2496 A251 line, and 4 more cells fed
by PPMT/IPMT), no regressions; m9-fin-pmt, IPMT_PPMT.xlsx and the CUMIPMT probes unchanged. Probes: +2 cases, 180 new
known-failure lines (172 last-digit cells and 8 visibly-off cells of the new cases: 7 unexplained pv-0 cancellations, T-2667, and the 2^-1023 discount, T-2666).
Not fixed: gen-m9 3c06f9298f65a3ba A109 is now 1 ulp (-8.888530001365962E-271 for Excel's ...961: visibly right, line kept);
gen-m9 a91bad4e84f49739 A158 (`PPMT(7,10,350,58827)`, a 2^-1023 discount Excel keeps) moves to T-2666. A fractional nper past
the range keeps the FV recurrence (gen-m9 0a957119475760c5 A153 stays bit-exact), T-2665.
