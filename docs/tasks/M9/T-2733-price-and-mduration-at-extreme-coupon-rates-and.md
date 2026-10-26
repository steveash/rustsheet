---
id: T-2733
title: "PRICE and MDURATION at extreme coupon rates and yields: the 3 visibly-off gen-m9 cells parked in deferred T-1374"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2732]
campaign: visible
---

## Goal
PRICE and MDURATION show Excel's 15 significant digits where both the coupon rate and the yield are far above 1. The
three gen-m9 cells below then pass at visible parity, and so does T-2732's sweep.

## Context
Gap triage 2026-10-25. T-1374 (`defer: phase-1b`) owns these cells, yet they are visibly off. Decision 0114 says a
deferred task with a visibly-off cell needs a campaign stage. Under a deferred owner, nothing would pick these cells
in Phase 1, so this task takes them and gives them a stage. T-1374 keeps its last-digit cells.
Cells (excel-graph, 2026-10-23; known-failure lines in eval/baseline/gen-m9.txt; Excel vs ours):
- gen-m9/0a957119475760c5 Calc!A279 `=PRICE(38136,44737,1E+300,7,100,2)`: 8.95449454935698e+300 vs 8.95449454935699e+300
  (one unit in the 15th digit).
- gen-m9/a91bad4e84f49739 Calc!A142 `=PRICE(43830,45293,100,CEILING.MATH(PERCENTILE.INC(Data!C2:D11,Data!C11),DATEDIF(100,8589,"M")),50,4,1)`
  (the yield argument is 278, T-1374): -133.76680342333566 vs -133.76680342333657 (7e-15 relative).
- gen-m9/30e20ba3302fea8b Calc!A90 `=MDURATION(39697,44449,CHOOSE(3,Data!D3,-Data!$E10,EDATE(58318,22)),DATEVALUE("15-Mar-2021"),2,Data!$A8)`
  (coupon about 58988, yield 44270): 5.029680055838246e-07 vs 5.029680055838228e-07 (3.6e-15 relative).
Not owned here, because the error is in an inner function:
- gen-m9/30e20ba3302fea8b Calc!A310 `PRICEMAT(..,FINV(1E-100,2,3),..)`: the inner F quantile is T-2511's.
- gen-m9/b6a02893a8d6b151 Calc!A260 `PRICEMAT(..,YIELD(..))`: the inner YIELD solver is T-1592's.
Re-check both after their owners land.
**Needs an oracle sweep (T-2732):** three points cannot fit an operation order. Its high-precision comparison sets
the stage. If the engine is the one far from the exact value, the stage is accuracy (a more accurate kernel). If
Excel is, the stage stays visible (reproduce Excel's algorithm; time-boxed, decision 0056).
Semantics: financial.md "PRICE and ODDFPRICE discount with Excel's `^`" (every ordinary PRICE cell passes with `^`).
probes/bond-huge-yield-overflow (T-2488) shows each argument extreme on its own, and all of it passes.
Code: crates/rustsheet-engine/src/functions/financial/financial_bonds.rs (PRICE, DURATION/MDURATION); Excel's `^` is
in numbers.md "Integer exponents: right-to-left square-and-multiply" and "Non-integer exponents: exp(y·ln x)".

## Acceptance criteria
- [ ] Visible parity: the three gen-m9 lines above disappear from eval/baseline/gen-m9.txt, and so do T-2732's PRICE,
      DURATION and MDURATION lines in eval/baseline/probes.txt that are visibly off, with no new known-failure lines in
      any baseline (`make eval-baseline`).
- [ ] financial.md records the rule found (or the candidates scored, if time-boxed) and cites the cells above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit residue of these functions (T-1374), PRICEMAT (above), YIELD and ODDFYIELD (T-1592, T-2007).

## Visible-parity stage
Decision 0083 and decision 0114: this task is done when every cell it owns is within half a unit of the 15th
significant digit of Excel's value. Criteria above that ask for bit-exact matches and are still unmet then move to a
follow-up task (no campaign key, `defer: phase-1b`), named in the Outcome. Regressions that stay within 4 ulp follow
decision 0083's allowance. Set the stage to `accuracy` instead of `visible` if T-2732's mpmath comparison shows Excel
close to correctly rounded and the engine not.
