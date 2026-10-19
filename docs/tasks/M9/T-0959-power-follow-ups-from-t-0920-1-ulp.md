---
id: T-0959
title: "POWER follow-ups from T-0920: 1-ulp odd roots, overflow error, odd-root tolerance probes"
milestone: M9
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
POWER and `^` match Excel on odd roots of negatives (bit for bit), overflow errors and the odd-root exponent tolerance.

## Context
Kernel map (T-1522, decision 0056): rank 13 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Review follow-ups recorded in T-0920 (2026-10-04): Excel gives -1.9999999999999998 for
`POWER(-8,1/3)` (POWER.xlsx Sheet1!D3, F9 and inside_a_lambda!C3/C4 are NumericUlp known failures),
we give -2. docs/excel-semantics/numbers.md "POWER and `^`" lists the UNVERIFIED parts: overflow
of a non-zero base (#NUM! vs #DIV/0!), the tolerance deciding that an exponent is 1/odd
(`POWER(-8,0.333333333333333)`), and `^` sharing the special cases. Code: `arithmetic::power`.

**T-1524 (2026-10-14):** the power kernel does not keep 64-bit intermediates. Every power sweep is best fitted
by doubles with Excel's twice-rounded operators (numbers.md "x87 precision is per operator, not per kernel").
The idealized x87 pow, the fyl2x/f2xm1/fscale sequence and 64-bit exp(y ln x) tie correctly rounded
(`power-nonint` 94 vs the model's 180). The one change found is T-1495's (twice-rounded products in
square-and-multiply). This task's acceptance criteria are unchanged by T-1524.

Note (planner, 2026-10-19): POWER.xlsx inside_a_lambda!C3/C4 are not `POWER(-8,1/3)`. They are the array `gelu(A1:C1)` /
`gelu(OFFSET(A1,0,0,1,3))` of the LAMBDA name `gelu` (TANH, SQRT, PI, `POWER(x,3)`), C3 is 0.34571400982514394 against
Excel's 0.3457140098251439: find which kernel gives the ulp before fitting POWER. T-1970 takes C3 out of the M2 function scope.

## Acceptance criteria
- [x] Probe case with oracle values for the three UNVERIFIED points; numbers.md updated.
- [x] POWER.xlsx NumericUlp cells pass or are explained in numbers.md.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Probes (excel-graph, 2026-10-19): probes-numeric/power-odd-root-tolerance (202 cells, `^` and POWER side by side),
  power-odd-root-tolerance-2 (134), power-ulp-sources (20), power-mirr-root (3). numbers.md "POWER and `^`" is VERIFIED:
  (a) overflow of a non-zero base is #NUM!; (b) a negative base takes an odd root when 1/y rounded to 15 significant digits
  is an odd integer below 2^31-1 (not the old 2^-48 relative tolerance: 26 cells changed verdict); (c) `^` and POWER agree on
  every probed special case.
- Non-integer powers now use EXP's kernel (`excel_exp`) on a twice-rounded product: all power probes pass (1,800+ cells,
  including `(-1E300)^(1/3)`). MIRR keeps the old libm `exp` root (`power_with_libm_exp`) because Mac desktop Excel's cached
  MIRR.xlsx Sheet2!N16 needs it, while the web gives the new `^` value (decisions 0030/0039; T-1075 item 13).
- POWER.xlsx inside_a_lambda!C3/C4 are TANH's ulp (`gelu` at 0.5), not POWER's: power-ulp-sources A4; noted on T-0917.
- Perf (math_lift, new cases, median of 11): `^` with a non-integer exponent now runs `excel_exp`'s F2XM1 series.
  "POWER, per row" (100,000 rows of `=A{r}^(1/3)`) 80 ms → 214 ms; "POWER, lifted" (`=SUM(A1:A1000000^0.37)`)
  0.85 s → 2.12 s (parent = libm `exp`, same machine, one case per process). Accepted for Excel bit parity
  (directive 1: `1E300^(1/3)` and the power sweeps); T-1945's fast path now covers `^` too.
- A workbook holding the literal 1E308 cannot be filled by excel-graph (already known); 9E307 stands in.
- Eval delta: gated suites unchanged (no fixes, no regressions); probes-numeric +357 new cells, 355 passing (power-mirr-root
  A1/A2 are the web side of the MIRR conflict) and 7 TANH cells of power-ulp-sources failing (T-0917).
