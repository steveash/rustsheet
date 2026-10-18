---
id: T-1374
title: "Bond, T-bill and depreciation values a few ulps off Excel (probes/fin-argument-modes base rows)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
The bond, T-bill, depreciation and cumulative-payment functions return Excel's bits for the probes/fin-argument-modes base rows.

## Context
YIELD's solver (T-1525, 2026-10-14): see `docs/excel-semantics/financial.md`, "YIELD stops short of PRICE's root";
probes/solver-iteration-traces Y!C:E (24 cells) and the basis-0 multi-coupon cells of yield-solver-sweep are scored
by `uv run rustsheet-fitlab yield` (90 cells, best candidate 2/90 bit-exact); the fit continues in T-1533. YIELD with one
coupon period left is closed form (T-1532).
probes/fin-argument-modes (excel-graph, 2026-10-11, T-1084). Base rows that miss:
- 1 to a few ulps off: CUMIPMT H72, DB H88, DDB H95, DURATION H117, INTRATE H139, MDURATION H163, ODDFPRICE H189,
  PRICE H251, YIELDDISC H352.
- Tens of ulps off: CUMPRINC H80 (about 37) and EFFECT H125 (about 32).
- Off at about 1e-10 to 1e-12 relative, so probably an iteration stopping point rather than an evaluation order: ODDFYIELD H200,
  YIELD H343. (TBILLEQ H313 and the other TBILL cells moved to T-1475 on 2026-10-13.)
- Added by T-1842: probes/fin-yield-odd-lift A1 and I5 (YIELD, about 1e-13 relative), A3 and I7 (ODDFPRICE, a few
  ulps), A5 and I9 (ODDFYIELD, about 1e-10 relative): the same base values as above, reached through the lift.
- Moved from T-1493 on 2026-10-13: YIELD A8-E8, A19-E19, A30-E30 and DURATION A20/C20 of
  probes/date-system-1900-financial-readers and its -shifted twin, and the 4 DURATION ulp cells of
  probes/coupdays-feb-month-end. probes/yield-solver-sweep (66 cells, excel-graph) is the YIELD sweep: Excel stops
  below the root, the stopping point does not move with the dates for the same day counts but does differ between
  two bonds with equal day counts and different YEARFRAC (docs/excel-semantics/financial.md, "YIELD stops short of
  PRICE's root").
- Added by T-1845: probes/fin-remaining-later-argument-ranges K11 `INDEX(CUMIPMT({0.01;0.02},360,100000,1,12,0),2)`
  (-23997.734423612783, ours an ulp off): the 12-period CUMIPMT sum, kept up to 32 periods by T-1845; the closed form
  past 32 periods is T-1890's.
The same DURATION, MDURATION and PRICE base values recur as the intersected controls of
probes/fin-remaining-later-argument-ranges I85-I103 and I165-I171 (T-1822), and so do the CUMIPMT, CUMPRINC, ODDFPRICE,
ODDFYIELD and YIELD ones there (I65-I84, I105-I136, I173-I184).
The annuity family (FV, PMT, PPMT, NPV) and the solvers (IRR, RATE, XIRR) are T-0901's and T-0916's, not this task's.
One probe point per function is not enough to fit a formulation: first add an oracle sweep per function family
(excel-probe skill; probes-numeric like T-0942), then fit. Split per family if larger than M.

## Acceptance criteria
- [ ] A probes-numeric sweep for the bond and depreciation families is oracle-filled.
- [ ] The last-digit cells of probes/discount-security-modes pass: INTRATE A1-A6, A11-A16, A19; YIELDDISC A68-A73, A78-A83, A86; PRICEMAT A103, A104, A106; YIELDMAT A113-A118, A123, A124, A126-A129, A131 (T-1624). Fitting hint: every INTRATE one-year cell is 0.10000000000000009 against Excel's 0.1, which suggests (redemption-investment)/investment rather than redemption/investment-1.
- [ ] The base rows above pass, or the Outcome records which formulation was ruled out.
- [ ] `make check` green; reviewers approve.

## Out of scope
- FV, PMT, PPMT, NPV (T-0901); IRR, RATE, XIRR, MIRR (T-0916).
