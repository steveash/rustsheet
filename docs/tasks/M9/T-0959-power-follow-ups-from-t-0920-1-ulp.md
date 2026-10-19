---
id: T-0959
title: "POWER follow-ups from T-0920: 1-ulp odd roots, overflow error, odd-root tolerance probes"
milestone: M9
lane: functions-math
status: todo
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
- [ ] Probe case with oracle values for the three UNVERIFIED points; numbers.md updated.
- [ ] POWER.xlsx NumericUlp cells pass or are explained in numbers.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
