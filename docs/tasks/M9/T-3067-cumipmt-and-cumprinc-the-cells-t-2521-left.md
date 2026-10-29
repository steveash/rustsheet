---
id: T-3067
title: "CUMIPMT and CUMPRINC: the cells T-2521 left (type 1 operation order of G2/H2, tiny-rate interest of both types, periods near 1E15)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-2521]
campaign: visible
---

## Goal
CUMIPMT and CUMPRINC are within half a unit of the 15th significant digit of Excel on the cells T-2521
left visibly off, or each one is documented in financial.md with the forms tried.

## Context
T-2521 found that Excel's short principal discounts with its own `L = ln(1+rate)` (`annuity::growth_log`, the
PPMT log) and that type 0's interest is the payments less the principal wherever the principal is no larger than
the interest (docs/excel-semantics/financial.md "CUMIPMT and CUMPRINC over many periods", T-2521 notes; probe
case probes/fin-cumipmt-gate-edges). Left, by cause:
- Type 1 from period 1 at ordinary rates: `CUMPRINC_CUMIPMT.xlsx` G2 (= fin-cumipmt-overflow-and-forms A7 and
  fin-cumipmt-gate-edges H5, H6) is `-pmt·m - H2` with `pmt = pv/((1+r)·(1-v^nper))·r` and Excel's own H2, and
  H2 (gate-edges G5) is to the bit with `-(pmt0·((1-v^m)/r)·v^(nper-end+1))` on Excel's EXP and `growth_log`
  (33 of the 54 type 1 principals probed to the bit, every other within 2 ulps) but that ordering misses gen-m9 a91bad4e84f49739 A193 by an
  ulp, where the kernel's order (divide the type 0 sum by `1+r`, libm `exp`) is to the bit. Find the one ordering
  (and EXP: Excel's or libm's) that keeps A193, A383, A398 and fits G5, G6.
- Type 1 over a long range: gen-m9 b73c6cc96e093f03 A62 `CUMPRINC(2,151,319288,3,42,1)` (40 periods) is to the bit
  with the geometric principal on `growth_log`, but past 32 periods the kernel takes the closed form (103 ulps off).
  The geometric sum over every long range moves fin-cumipmt-period-bound A6, A11 and A30 (1,024 to 99,998 periods)
  off the bit: probe where between 40 and 1,024 periods Excel's sum changes form.
- `GEOMETRIC_MIN_DISCOUNT` (3.6) is a fit between gen-m9 A300 (exponent 2E-7) and gate-edges C1/D1 (3.62):
  probe exponents 0.5 to 3.5 and move it to the evidence (fin-argument-modes H80 at 3.58 is 4 ulps off with the
  per-period sum and 2 with the geometric one).
- Type 0 interest where the principal is larger than the interest at tiny rates: fin-cumipmt-gate-edges A2-A8
  (`CUMIPMT(1E-4,60,100000,3,10,0)`): `-pmt·m - P` with P one ulp off moves the result by 128-512 ulps; neither the
  balance form nor any principal form tried is exact on all.
- Type 1 interest at tiny rates: fin-cumipmt-gate-edges E1-E10 on both sides of `TINY_RATE_GROWTH` and gen-m9
  e168be7f6d3f84c2 A379 `CUMIPMT(1E-9,235,379138,7,14,1)`: Excel's value is not `-pmt·m - CUMPRINC` with its own
  CUMPRINC (F1-F10) for any payment form tried.
- 79d87fc527927626 A352 `CUMIPMT(1E-15,283,(1.005+3.14159+1E15)%,2,16,0)`: 3 ulps of `pmt·m`.
- Periods near 1E15 (untouched by T-2514 and T-2521): fin-remaining-later-argument-ranges K3,
  fin-cumipmt-overflow-and-forms A9-A11, fin-cumipmt-period-bound A18 and A6.
Code: functions/financial/cumulative.rs.

## Acceptance criteria
- [ ] Each cell above is visibly right (no new known-failure lines, `make eval-baseline`), or it is documented in
      financial.md with the forms tried.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit-exact fits of cells that are already visibly right (T-1374).

## Outcome (partial, 2026-10-28; the task stays open)
The near-1E15 cells (A9-A11, K3) were fitted in float64 against 40 forms and are not solved. What was tried, and why
the best form (the geometric sum on `growth_log`, within an ulp of all three) was not kept, is in financial.md "T-3067
notes". Next: probe the near-1E15 principals directly (CUMPRINC at m = 1 to 12 and at rates 0.001, 0.01, 0.1) so
Excel's discount error can be measured per period instead of inferred from three cancelled interests. Nothing else
in the list above was started.
