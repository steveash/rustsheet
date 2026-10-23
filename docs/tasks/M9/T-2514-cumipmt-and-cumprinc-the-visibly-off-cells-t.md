---
id: T-2514
title: "CUMIPMT and CUMPRINC: the visibly-off cells T-2363 left (Excel's power on integer rates, type 1 principal and tiny-rate interest, periods near 1E15)"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2363]
campaign: visible
---

## Goal
CUMIPMT and CUMPRINC are within half a unit of the 15th significant digit of Excel on the cells T-2363
left visibly off.

## Context
T-2363 made type 0's interest Excel's `-pmt·m - principal` (balance form kept where well-conditioned) and
fixed 18 cells; docs/excel-semantics/financial.md "CUMIPMT and CUMPRINC over many periods" (T-2363 notes)
has the fit. Left visibly off, by cause:
- Integer-rate principals, type 0 (Excel's power is not `exp(m·log1p r)`: 1 ulp off exact for base 2,
  311 ulps for base 11; no libm `pow`, `exp2·log2`, repeated-product or squaring form fits): gen-m9
  9a1fa5366bc50918 A18 `CUMPRINC(4,114,114041,4,13,0)` and A361 `CUMPRINC(10,261,484823,4,24,0)`,
  67a4c03afd63f988 A110 `CUMPRINC(6,82,372488,5,16,0)`, 1cd3096e3e87c965 A147 `CUMPRINC(1,200,480533,5,18,0)`,
  d0854366830dcdee A98 `CUMPRINC(10,230,351888,4,21,0)` (11 to 18 ulps).
- Type 1 principals over short ranges (closed form; the per-period form is no closer): 5ded7594e597e31e
  A22, A267; b6a02893a8d6b151 A398; a91bad4e84f49739 A193; 0a957119475760c5 A383; 9d9ceba257f8001b A372
  (`CUMPRINC(0.1,1023,478581,7,24,1)`, 81 ulps); b73c6cc96e093f03 A62; CUMPRINC_CUMIPMT.xlsx H26.
- Type 1's interest at tiny rates: Excel is `-pmt1·m - P` (b72d952c5f52cb4d A257 to the bit with
  `pmt1 = pv·r/(1-v^nper)/(1+r)`), ours is the IPMT sum; switching type 1 to that form also gets G2 (= probes
  A7) within 4 ulps but moves G3, G9, G15, G12, G16, G17, G25 and probes A8 from bit-exact to 1-3 ulps
  (decision 0083 allowance). e168be7f6d3f84c2 A379 `CUMIPMT(1E-9,235,379138,7,14,1)` fits neither;
  6e18550e9fc9c1b9 A329 (STDEVA over a type 1 CUMIPMT) follows.
- 79d87fc527927626 A352 `CUMIPMT(1E-15,283,1E13,2,16,0)` -0.1456298828125: the `-pmt·m - P` cancellation
  needs the principal to the bit (ours 3 ulps of 5.3E11 off).
- 01523d247594952e A322 `DOLLARDE(CUMPRINC(1E-300,129,257125,1,17,0),16)`: CUMPRINC at a rate lost in
  `1 + rate` is probably the balance drop T-2363 gave the interest (unchecked).
- Periods near 1E15: probes fin-remaining-later-argument-ranges K3, fin-cumipmt-overflow-and-forms A9-A11,
  fin-cumipmt-period-bound A18 (type 1), and A6 (1E-15 past 2^53; see T-2363's notes on a double-double
  exponent).
- T-2363 moved two of these slightly further from Excel without a new baseline line: probes A10 (main
  ...8880, now ...8903, Excel ...835) and A352 (main -0.14573172, now -0.14581298828125).
- T-2363's balance-form band [2^-10, 2^10] on the growth `(1+r)^(end-1) - 1` is a fit with no oracle cell near
  either edge (nearest: A392 at 1.7E-8, A189 at 1.1E7): probe cells that bracket both edges.
- 3c06f9298f65a3ba A91 is POISSON.DIST's (the rate is `1/POISSON.DIST(10,42,1)`, ours 1E-15 relative off);
  not CUMIPMT's.
Code: functions/financial/cumulative.rs. Fit scaffolding: rebuild the arguments of every CUM cell
(T-2363 used a scratch suite whose cells evaluate each argument) and score forms in float64.

## Acceptance criteria
- [ ] Visible parity: the cells above (but A91) are within half a unit of the 15th significant digit of
      Excel's value, with no new known-failure lines in any baseline (`make eval-baseline`), or the ones that
      cannot be fitted are documented in financial.md with the forms tried.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit-exact fits once the cells are visibly right (T-1374).
- POISSON.DIST (A91).
