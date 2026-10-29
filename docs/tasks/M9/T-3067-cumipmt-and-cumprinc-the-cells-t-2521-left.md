---
id: T-3067
title: "CUMIPMT and CUMPRINC: the cells T-2521 left (type 1 operation order of G2/H2, tiny-rate interest of both types, periods near 1E15)"
milestone: M9
lane: functions-financial
status: todo
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

## Outcome (partial, run 2, 2026-10-29; the task stays open)
Probed the near-1E15 principals directly, as the run 1 note suggested (probes/fin-cumprinc-near-1e15: CUMPRINC over
the last 1 to 12 periods at three rates, the PPMTs, CUMIPMT and type 1). The best of 30 forms, the balance drop on
Excel's EXPM1 and `growth_log`, is exact on 23 of 36 principals and within an ulp on the rest. In the kernel it
takes 8 to 14 bit-exact cells off, so nothing in the kernel changed. The details and the next probe (nper 1E6 and
1E10) are in financial.md "T-3067 notes". Eval delta: one new probe case, 84 cells, 44 passing; the 40 failing
cells are new known failures; no regressions.

## Outcome (partial, run 3, 2026-10-29; the task stays open)
Probed where Excel's CUMPRINC changes form (probes/fin-cumprinc-form-boundaries, 132 cells): it is the leading
discount `(nper-end+1)·L` that decides, not the range's length or its deepest discount. The geometric principal now
takes a short range from a leading exponent of 0.3 (`GEOMETRIC_MIN_LEAD`, replacing `GEOMETRIC_MIN_DISCOUNT`) and a
long range from 3.6 (`LONG_GEOMETRIC_MIN_LEAD`), which closes the "type 1 over a long range" (A62) and
`GEOMETRIC_MIN_DISCOUNT` items above. Eval delta: 7 known failures fixed (gen-m9 A62; CUMPRINC_CUMIPMT.xlsx H9, H10,
H15, H21; gate-edges G9, G11), 0 regressions; the new case is visibly right on all 132 cells, 73 to the bit (59 new
known failures, each within 2 ulps). Left: type 1 from period 1 (G2/H2), the tiny-rate interests (gate-edges A2-A8,
E1-E10, A379), A352 and the cells near period 1E15 (financial.md "T-3067 notes").

## Outcome (partial, run 4, 2026-10-29; the task stays open)
Measured how Excel builds the interest (probes/fin-cumipmt-tiny-rate-structure, 226 cells): CUMIPMT is exactly its own
PMT times the period count less its own CUMPRINC, and a type 1 range from period 1 is built from the range from period
2 (`CUMPRINC = PMT + CUMPRINC(2..e)`, `CUMIPMT = PMT·(e-1) - CUMPRINC(2..e)`), to the bit on all 82 probed relations.
So the tiny-rate interests (gate-edges A2-A8, E1-E10, A379) and G2/H2 are visibly off only through an ulp of PMT or
CUMPRINC; the engine's PMT is an ulp off Excel's at those rates (T-3014), so taking the rule now takes 29 bit-exact
cells off and nothing in the kernel changed. Follow-up: T-3118 (the rule, after T-3014). Left here: CUMPRINC's own
bits (the best type 1 form on Excel's PMT is exact on 22 of 58), A352 and the cells near period 1E15 (financial.md
"T-3067 notes"). Eval delta: one new probe case, 226 cells, 66 passing; the 160 failing cells are new known failures
(68 visibly off, all interests that ride on PMT's or CUMPRINC's ulps); no regressions.
