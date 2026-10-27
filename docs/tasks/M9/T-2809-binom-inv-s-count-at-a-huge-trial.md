---
id: T-2809
title: "BINOM.INV's count at a huge trial count (Excel 1073741823 at 2^31 - 2 trials)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
BINOM.INV and CRITBINOM give Excel's count at a trial count near the 2^31 - 1 limit.

## Goal note
`BINOM.INV(2147483646,0.5,0.5)` is 1073741823 in Excel and 1073744668 here, 2845 counts out; the
same at 2147483646.9 (the trials truncate first).

## Context
Found by T-2435's probe while settling the trial limit itself (docs/excel-semantics/statistics.md
"BINOM.INV and CRITBINOM take BINOM.DIST's trial limit (T-2435)"). The search is a bisection on
statrs' `Binomial::cdf` (functions/statistical/binom.rs `binom_inv_of`), which at n = 2^31 is far
enough off at the median to move the crossing; BINOM.DIST's own cdf there is the double-double walk
of `discrete.rs`, whose budget stops well below such an n (`discrete.rs` comment: Excel's limit is
between 2E9 and 1E10, T-2429). Decide which cdf the search should use at a huge n before changing
the search.
Cells: probes/t2435-dist-extremes A15 and A18 (known failures).

## Acceptance criteria
- [ ] probes/t2435-dist-extremes A15 and A18 pass, with no new known-failure lines in any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The trial limit itself (T-2435, done).
