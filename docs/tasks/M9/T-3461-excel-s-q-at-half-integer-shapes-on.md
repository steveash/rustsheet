---
id: T-3461
title: "Excel's Q at half-integer shapes on [a + 1, 30]: the cells T-3285's correctly rounded kernel moved off Excel's bits"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3285]
defer: phase-1b
---

## Goal
Find Excel's own form for Q(a, x) at a half-integer shape a from 3/2 to 20 and a + 1 <= x <= 30 (CHISQ.DIST.RT, CHIDIST,
CHISQ.TEST at an odd df), so the cells T-3285 moved off Excel's bits match again, without a fitted cut.

## Context
- T-3285 takes Q there correctly rounded (`incomplete_gamma/half_integer.rs`); decision 0202 (owner ruling) accepted the
  gated cells this cost under decision 0083's ulp allowance, whose condition 4 needs this task to name them. Main
  against T-3285's head, ulps of Excel's value:
  - lost: gen-m9/fec39efaba9284c3 Calc!A249 `CHIDIST(42,7)` = Q(3.5, 21), 0 to 1 (Excel +1 ulp from the truth);
  - farther: gen-m9/320d51765af9b6b7 Calc!A249 and 7674c89c824aca43 Calc!A332 `CHIDIST(9,5)` = Q(2.5, 4.5), 2 to 3
    (Excel -2.6 from the truth); gen-m9/b8ee55934c5b824c Calc!A262 `CHIDIST(37.5,7)`, 2 to 3, and Calc!A400
    `CHIDIST(37.5,5)`, 1 to 2.
  - Ungated (probes-numeric): t3284-gamma-deep-upper-tail A245 `CHISQ.DIST.RT(42,7)` and A252 `(54,9)` 0 to 1.
- Excel is within a few ulp of the truth there but rarely correctly rounded: on T-3285's fresh sweep
  (probes-numeric/t3285-chisq-dist-rt-half-integer-sweep, docs/research/t3285) it is the correctly rounded double at 5
  of 228 cells. Forms already ruled out (statistics.md "Incomplete gamma: accurate kernel (T-0971)"): Excel's ERFC plus
  the finite series, in doubles or double-double. T-3269's Stirling ratio form fits past x = 30; whether the same form
  holds below 30 at half-integer shapes is the first lead.
- Phase 1b under decision 0188 R5: every cell here is within 15 digits of Excel and within Gate A's envelope.

Cells this task owns: the gated and ungated cells listed above.

## Acceptance criteria
- [ ] The listed cells match Excel, or a probe shows why not (recorded in statistics.md).
- [ ] Any form adopted passes R4 (hold-out split, fresh sweep, FITTED tags) and no cell regresses (main-vs-HEAD).
- [ ] `make check` green; reviewers approve.

## Out of scope
- x > 30 (T-3269). Integer shapes.
