---
id: T-2032
title: "Discount securities last digits: DISC, PRICEDISC, INTRATE, YIELDDISC, PRICEMAT, YIELDMAT (split from T-1374)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
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
- [ ] The DISC.xlsx and INTRATE.xlsx F lines above leave eval/baseline/ironcalc-calc.txt, and the
      discount-security-modes, date-system-*-daycount, date-system-1904-financial and fin-argument-modes
      H139/H352 F lines above leave eval/baseline/probes.txt (each removed line listed in the PR); no
      new F lines anywhere. Lines that resist stay, listed in the Outcome with expected/actual.
- [ ] Each function's formulation is in docs/excel-semantics/financial.md, citing the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TBILLPRICE, TBILLYIELD, TBILLEQ (matched by T-1475); bond functions (PRICE, YIELD, DURATION; T-1374, T-1592).
- Argument-mode and error-order cells of the same functions.
