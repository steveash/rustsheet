---
id: T-2032
title: "Discount securities last digits: DISC, PRICEDISC, INTRATE, YIELDDISC, PRICEMAT, YIELDMAT (split from T-1374)"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
DISC, PRICEDISC, INTRATE, YIELDDISC, PRICEMAT and YIELDMAT return Excel's bits: the closed forms are
right, the operation order and the way the day count enters (a year fraction, or basis and day count
separately) are not.

## Context
Split out of T-1374 (bond, T-bill and depreciation last digits) by gap triage 2026-10-19, which asks
for one task per family. About 73 failing cells, all NumericUlp or NumericClose (a few to ~40 ulp):
- ironcalc-calc/FINANCIAL/DISC.xlsx F2, F6, F10, F15, F19 and ironcalc-calc/FINANCIAL/INTRATE.xlsx
  F2-F6 (eval/baseline/ironcalc-calc.txt; Excel-cached). Example: INTRATE F2 (basis 0, investment
  2000000, redemption 2040000, 90 days) is exactly 0.08 in Excel.
- probes/discount-security-modes (excel-graph, T-1624): INTRATE A1-A6, A11-A16, A19, A21; YIELDDISC
  A68-A73, A78-A83, A86, A88; PRICEDISC A43; PRICEMAT A103, A104, A106; YIELDMAT A113-A118, A123,
  A124, A126-A129, A131.
- probes/date-system-1900-financial-daycount A1, A2, A4 (DISC), A19 (INTRATE), A21 (YIELDDISC),
  A23, A24 (PRICEMAT), A25 (YIELDMAT); probes/date-system-1904-financial-daycount A19-A21, A25;
  probes/date-system-1904-financial A25, A27; probes/fin-argument-modes H139 (INTRATE), H352 (YIELDDISC).
Leads (scratch fits during triage, Python doubles; not yet tried in the engine):
- DISC: ours is `(1 - pr/redemption) / yearfrac`. `(1 - pr/redemption) * (B/DSM)`, B the basis year
  length and DSM the basis day count, gives all three failing basis-0/4 DISC.xlsx cells (F2, F15,
  F19); `(1 - pr/redemption) / (DSM/B)` and the `(redemption - pr)/redemption` forms give at most 2.
  Check it on the basis 1-3 cells, which pass today.
- INTRATE: ours is `(redemption/investment - 1) / yearfrac`; `(redemption - investment)/investment`
  times B/DSM (or divided by DSM/B) gives F2's exact 0.08, `redemption/investment - 1` gives
  0.08000000000000007. T-1374 noted that every one-year INTRATE cell of discount-security-modes is
  0.10000000000000009 for us against Excel's 0.1, the same sign.
- YIELDDISC is `(redemption/pr - 1)/yearfrac` today; PRICEMAT/YIELDMAT combine three day counts
  (discount.rs lines with `dim`, `dsm`, `a`): fit their grouping after the simpler three.
Code: crates/rustsheet-engine/src/functions/financial/discount.rs (`discount_year_fraction` reads
YEARFRAC through `get_yearfrac`); the basis day counts live with YEARFRAC. Use the fit lab or a scratch
script over the cells above; record the formulation per function in docs/excel-semantics/financial.md.
If a function's cells do not separate the candidates, say so in the Outcome and file a probes-numeric
sweep for it (research lane) rather than guessing.

## Acceptance criteria
- [x] The DISC.xlsx and INTRATE.xlsx F lines above leave eval/baseline/ironcalc-calc.txt, and the
      discount-security-modes, date-system-*-daycount, date-system-1904-financial and fin-argument-modes
      H139/H352 F lines above leave eval/baseline/probes.txt (each removed line listed in the PR); no
      new F lines anywhere. Lines that resist stay, listed in the Outcome with expected/actual.
- [x] Each function's formulation is in docs/excel-semantics/financial.md, citing the cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- TBILLPRICE, TBILLYIELD, TBILLEQ (matched by T-1475); bond functions (PRICE, YIELD, DURATION; T-1374, T-1592).
- Argument-mode and error-order cells of the same functions.

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 34 in total.
- INTRATE: 7 cells (13-14 digits right 7, 10-12 0, 6-9 0, fewer or 0 vs nonzero 0); most in `probes/discount-security-modes.case.json` 3; `ironcalc-calc/FINANCIAL/INTRATE.xlsx` 2. e.g. `INTRATE(A2,B2,C2,D2,E2)` Excel 0.08, ours 0.08000000000000007
- YIELDDISC: 14 cells (13-14 digits right 14, 10-12 0, 6-9 0, fewer or 0 vs nonzero 0); most in `probes/discount-security-modes.case.json` 12; `probes/date-system-1904-financial-daycount.case.json` 1. e.g. `YIELDDISC(43101,43466,95,100)` Excel 0.05263157894736842, ours 0.05263157894736836
- YIELDMAT: 13 cells (13-14 digits right 13, 10-12 0, 6-9 0, fewer or 0 vs nonzero 0); most in `probes/discount-security-modes.case.json` 11; `probes/date-system-1904-financial-daycount.case.json` 1. e.g. `YIELDMAT(43101,43466,42736,0.06,98)` Excel 0.07692307692307698, ours 0.0769230769230771

Against a 50-digit reference, Excel is close to correctly rounded for this family and the engine is the inaccurate side, so an accurate kernel (not a fit) should reach this stage.

## Outcome
The kernels now take YEARFRAC's two parts, the basis day count and the basis year length
(`Model::year_fraction_parts`, which YEARFRAC itself now divides), instead of the year fraction, and round in
Excel's order: DISC `(1 - pr/redemption) * (B/DSM)`, INTRATE `(redemption - investment)/investment * (B/DIM)`,
YIELDDISC `(redemption - pr)/pr * (B/DSM)`, PRICEDISC `redemption - discount*redemption*DSM/B`, and PRICEMAT and
YIELDMAT from `getMatFactors` parts (B of issue to settlement, A, DIM, DSM = DIM - A), YIELDMAT as
`((1 + DIM/B*rate) - T)/T * (B/DSM)`. Recorded in financial.md "Discount securities: day count and year length".
Eval delta: 73 known failures fixed (ironcalc-calc 916 -> 906: DISC.xlsx F2, F6, F10, F15, F19, INTRATE.xlsx F2-F6;
probes 1632 -> 1569: discount-security-modes, date-system-1900/1904-financial-daycount, date-system-1904-financial
A25, A27, fin-argument-modes H139, H352), 0 regressions, visibly-off cells 1750 -> 1716 (baselines rewritten by `make eval-baseline`). No cell resisted.
RECEIVED is regrouped to `discount*(DIM/B)`, the same number as before; no oracle cell separates RECEIVED's
candidate groupings, and none fails, so no sweep is filed for it.
