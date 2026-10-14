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

## Acceptance criteria
- [ ] Probe case with oracle values for the three UNVERIFIED points; numbers.md updated.
- [ ] POWER.xlsx NumericUlp cells pass or are explained in numbers.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
