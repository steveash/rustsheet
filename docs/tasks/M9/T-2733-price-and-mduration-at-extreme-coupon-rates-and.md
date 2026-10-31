---
id: T-2733
title: "PRICE and MDURATION at extreme coupon rates and yields: the 3 visibly-off gen-m9 cells parked in deferred T-1374"
milestone: M9
lane: functions-financial
status: done
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
**T-2732's sweep** (excel-graph, 2026-10-26):
probes/bond-extreme-coupon-yield: this task owns every failing cell of the case (414 known-failure lines in eval/baseline/probes.txt; 62 of them visibly off).
By function, the visibly-off cells (Excel vs ours; layout in the case description):
- PRICE, 43: Calc!A8, A14, A20, A26, A32, A38, A44, A45, A62, A68, A75, A87, A93, A99, A105, A122, A123, A134, A135,
  A140, A141, A181-A186, A188-A198, and the coupon term alone at redemption 1E-300 Parts!F1, F2, F5, F6, G2. For
  example Calc!A20 `PRICE(38136,44737,1E+10,7,100,1,0)` 55709579072.88489 vs 55709579072.88501, Calc!A184
  `PRICE(43830,45293,10,42,100,4,1)` 15.085723843923432 vs 15.085723843923375, Calc!A193
  `PRICE(39697,44449,1E+4,1E+3,100,2,0)` -52532.50712345407 vs -52532.507123453775.
- DURATION, 8: Calc!B27, B182, B185, B186, B188, B189, B191, B192. For example Calc!B186
  `DURATION(43830,45293,10,1E+3,4,1)` 0.006434782608695652 vs 0.006434782608695676.
- MDURATION, 11: Calc!C102, C183, C186, C189, C192-C198. For example Calc!C193 `MDURATION(39697,44449,1E+4,1E+3,2,0)`
  2.417387447327567e-05 vs 2.4173874473275588e-05.
The other 352 failing cells are within 15 digits (319 within 4 ulp), and stay this task's only until it lands: bit-exact
leftovers then go to the follow-up named in the Outcome.
**Stage (T-2732's mpmath comparison, financial.md "PRICE and ODDFPRICE discount with Excel's `^`"):** stays `visible`.
On the short stubs (rows 182-198: DSC/E 2/92 and 4/180) Excel is close to correctly rounded and the engine is not
(DURATION and MDURATION within 3.1 ulp in Excel, 5 to 30 ulp in the engine), so a more accurate duration kernel fixes
those. On the main grid (DSC = 26 days) Excel's PRICE is itself up to 32 ulp off and the engine is closer in 13 of 29
visible cells, so only Excel's operation order gives its digits there. The pieces (COUP*, `(1+yld/f)^t`, the
redemption term) all pass: the gap is in the coupon sum.
**Oracle sweep (T-2732, done):** three points could not fit an operation order, so T-2732 swept the region; its high-precision comparison set
the stage (above). The rule it applied: if the engine is the one far from the exact value, the stage is accuracy (a more accurate kernel). If
Excel is, the stage stays visible (reproduce Excel's algorithm; time-boxed, decision 0056).
Semantics: financial.md "PRICE and ODDFPRICE discount with Excel's `^`" (every ordinary PRICE cell passes with `^`).
probes/bond-huge-yield-overflow (T-2488) shows each argument extreme on its own, and all of it passes.
Code: crates/rustsheet-engine/src/functions/financial/financial_bonds.rs (PRICE, DURATION/MDURATION); Excel's `^` is
in numbers.md "Integer exponents: right-to-left square-and-multiply" and "Non-integer exponents: exp(y·ln x)".

## Acceptance criteria
- [x] Visible parity: the three gen-m9 lines above disappear from eval/baseline/gen-m9.txt, and so do T-2732's PRICE,
      DURATION and MDURATION lines in eval/baseline/probes.txt that are visibly off, with no new known-failure lines in
      any baseline (`make eval-baseline`).
- [x] financial.md records the rule found (or the candidates scored, if time-boxed) and cites the cells above.
- [ ] `make check` green (done); reviewers approve (pending: the agent panel has not run on this head).

## Out of scope
- Last-digit residue of these functions (T-1374), PRICEMAT (above), YIELD and ODDFYIELD (T-1592, T-2007).

## Visible-parity stage
Decision 0083 and decision 0114: this task is done when every cell it owns is within half a unit of the 15th
significant digit of Excel's value. Criteria above that ask for bit-exact matches and are still unmet then move to a
follow-up task (no campaign key, `defer: phase-1b`), named in the Outcome. Regressions that stay within 4 ulp follow
decision 0083's allowance. Set the stage to `accuracy` instead of `visible` if T-2732's mpmath comparison shows Excel
close to correctly rounded and the engine not.

## Outcome
- `CouponSchedule::price` (PRICE, and YIELD's Newton): accrued `rate/f·(A/E)·100`, and coupon k discounted at
  `k + DSC/E` from k = 0. `duration_of` (DURATION, MDURATION): the same coupon exponent. Rule and fit table:
  financial.md "PRICE, DURATION and MDURATION sum order at a huge coupon rate and yield".
- Eval result on probes/bond-extreme-coupon-yield (`rustsheet-eval show`): 532 of 579 cells pass; all 240 PRICE
  cells are bit-exact, and the 47 cells left are all DURATION or MDURATION (Calc columns B and C), within 2 ulp. The
  offline fit that ranked the candidate orders is in the financial.md entry; it is not the eval result.
- Eval (`update-baseline` after merging `main` at 1248554ae): probes known failures 2316 → 1933 (387 fixed: 371 in
  bond-extreme-coupon-yield, 16 in the yield-* probes), gen-m9 3015 → 2999 (17 fixed, among them the three gen-m9
  cells of this task). Every failing cell of probes/bond-extreme-coupon-yield (47, all numeric) is within half a unit
  of Excel's 15th significant digit; no baseline gained a non-numeric or visibly-off line.
- Regressed, recorded with `update-baseline --allow-regressions --suite probes --suite gen-m9` under decision 0083's
  standing ulp-regression allowance (numeric, ≤ 4 ulp, none visibly off, 404 fixed against 5 regressed), owned by
  T-2973 (Excel vs ours, ulp):
  - probes/bond-extreme-coupon-yield Calc!B7 `DURATION(38136,44737,100,1,1,0)`: 1.0721863067796484 vs
    1.0721863067796482, 1 ulp;
  - Calc!B43 `DURATION(38136,44737,100,1,2,0)`: 1.0722166274036073 vs 1.0722166274036076, 1 ulp;
  - Calc!C7 `MDURATION(38136,44737,100,1,1,0)`: 0.5360931533898242 vs 0.5360931533898241, 1 ulp;
  - Calc!C43 `MDURATION(38136,44737,100,1,2,0)`: 0.7148110849357382 vs 0.7148110849357384, 2 ulp;
  - gen-m9/5ded7594e597e31e Calc!A123 `DURATION(37952,44607,123456789.123,Data!$C3,1,2)`: 2.1997586986000424 vs
    2.199758698600043, 1 ulp.
  probes/yield-newton-domain P!B7, which regressed by an ulp before the merge, passes on top of T-3215's YIELD start.
- Follow-ups: T-2972 (the 43 DURATION/MDURATION residue cells, `defer: phase-1b`), T-2973 (the five regressed cells).
