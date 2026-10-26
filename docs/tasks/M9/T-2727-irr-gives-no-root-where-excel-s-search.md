---
id: T-2727
title: "IRR gives no root where Excel's search gives up: far and near -1 roots are #NUM!, a root near 1E+15 is #DIV/0! (gen-m9, 11 cells)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
IRR returns Excel's `#NUM!` where Excel's search finds no root but ours (the fitted secant or its Newton/bisection
fallback) converges, and Excel's `#DIV/0!` for the flows where Excel gives it.

## Context
Cut from T-2494 on 2026-10-25 (its 1E+300-guess cells were fixed there). Probes (excel-graph, 2026-10-25):
- probes/irr-guess-edges-2 A15-A34: `IRR({-1,X})` is `#NUM!` for X = 1E+13, 1E+14, 2E+14, 5E+14, 9E+14, 2E+15 and 1E+16
  (ours converges on X-1, the exact root: the two-flow NPV is linear in v, so the secant lands on it in one step),
  but `#DIV/0!` for X = 1E+15 (also with guesses 1E+15 and 5E+14, A28, A29); `IRR({-1E-15,1})` and `IRR({-1E-13,1})` are
  `#NUM!` (ours 1.0008E+15, 9.997E+12). Known good: `IRR({-0.0001,2,42})` = 20019.97799564059 and
  `IRR({-2.5,-1,123456789.123})` = 7026.083662734896 (financial.md "IRR is a secant in v"), so the bound on far roots
  lies between about 2E+4 and 1E+13, or the rule is not a bound on r.
- probes/irr-guess-edges A27-A29: `IRR(D!A1:D12)` over gen-m9 bc04c1f478c9896c's Data!A1:D12 is `#DIV/0!` with guesses 0.05,
  0.1 and 0.5 (ours `#NUM!`); `IRR({0,1E+15,2,42,1E+15,-0.0001})` (its first numbers) is `#NUM!` as ours. A37: `IRR({1E+15,1,-1E+15},0.05)` is
  `#NUM!` (ours -4.8E-16). A38, A39:
  `IRR({-1,1E+15},0.05)` and `IRR({-1,1E+300})` are `#DIV/0!`.
- The remaining cells have roots near -1 (ours -0.99995, -0.9998, -0.99998, -0.98958, -0.9999999999) or around -0.75 to
  -0.88 that Excel does not return. Read financial.md "IRR is a secant in v = 1/(1 + r)" and "The IRR and XIRR solver caps";
  T-1686 (the fallback) and T-2565 (Excel's steps far from the root) are related probes.
Code: `compute_irr`, `irr_secant`, `irr_fallback` in crates/rustsheet-engine/src/functions/financial/financial_util.rs.

Owned gen-m9 cells (T-2257 triage and T-2436 probe, excel-graph): 11 known-failure lines in eval/baseline/gen-m9.txt.
- gen-m9/0a957119475760c5 Calc!A293 `=IRR(Data!B10:E12,1)`: Excel #NUM!, ours -0.9999500028653313
- gen-m9/1b60bf99d71424e7 Calc!A117 `=IRR(Data!A4:D12,N(Data!D3))`: Excel #NUM!, ours -0.8810894616040402
- gen-m9/1cd3096e3e87c965 Calc!A277 `=IRR(Data!A10:C12,ROWS(..))`: Excel #NUM!, ours -0.7535582803865576
- gen-m9/3c06f9298f65a3ba Calc!A251 `=IRR(Data!B3:E9,(-0.9999999999))`: Excel #NUM!, ours -0.9999999998928571
- gen-m9/5c6a23c4f6597f1c Calc!A104 `=IRR(Data!B4:E6,10)`: Excel #NUM!, ours -0.9895760741004859
- gen-m9/5c6a23c4f6597f1c Calc!A349 `=IRR(Data!C9:D12)`: Excel #NUM!, ours -0.8599450553923559
- gen-m9/9a1fa5366bc50918 Calc!A400 `=IRR(Data!A1:C3)`: Excel #NUM!, ours 31622776.086705893
- gen-m9/abb7d565af89b849 Calc!A254 `=IRR(Data!A9:D10)`: Excel #NUM!, ours -0.9999803488388107
- gen-m9/c0681a8b150ce301 Calc!A22 `=IRR(Data!A1:C9)`: Excel #NUM!, ours -0.9997999999463046
- gen-m9/abc2939ebd1e2365 Calc!A280 `=FISHERINV((COUNTIF(..)/IRR(Data!C10:D12,Data!E12)))`: Excel #NUM!; inner IRR ours 19999998.999885842
- gen-m9/bc04c1f478c9896c Calc!A56 `=IRR(Data!A1:D12,0.05)`: Excel #DIV/0!, ours #NUM!

## Acceptance criteria
- [ ] Visible parity (gen-m9): these 11 known-failure lines disappear from eval/baseline/gen-m9.txt (`make eval-baseline`),
      with no new known-failure lines in any baseline, and the probes/irr-guess-edges and irr-guess-edges-2 IRR cells above pass.
- [ ] The rule is recorded in docs/excel-semantics/financial.md with probe evidence; unit tests for a far root, a root near -1
      and the `#DIV/0!` case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IRR's returned digits (T-2131); the 1E+300-guess `#VALUE!` (done in T-2494).

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type or error, so the task is in the campaign.
