---
id: T-3370
title: "Oracle probe: T.INV near p = 1/2 (centre quantile, follow-up of T-2724)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence pins T.INV for p near 1/2 (a tiny quantile at the centre) and for the band between c = |2p - 1| =
0.075 and 0.1, so the T-2724 centre path for one-tailed T.INV is backed by oracle cells, not only by mpmath.

## Context
T-2724 (PR #2510) added a centre path (`t_centre_quantile` in crates/rustsheet-engine/src/functions/statistical/
t_dist.rs) for c = P(|T| < t) below `T_CENTRE_C` = 0.075. For TINV and T.INV.2T near p = 1, oracle cells confirm Excel
is the true quantile there. For T.INV at p = 0.5 +- small, there is no oracle cell: the unit test
`centre_quantiles_keep_their_digits` checks T.INV(0.5+1E-10,5) against mpmath (2.6343057421036885E-10). The
excel-fidelity review of #2510 flagged this as unconfirmed (MINOR). docs/excel-semantics/statistics.md "Student t
sweep" ("Centre quantiles are accurate") states the rule. The T-3061 entry says Excel is the rounded-y form at
q = 0.45 and the exact quantile at q = 0.475; the switch between them has not been located.

Probe with the `excel-probe` skill (a probes-numeric case), at df 2, 5, 30, 1000 and 1E6:
- `T.INV(0.5+1E-10,df)`, `T.INV(0.5-1E-10,df)`, `T.INV(0.5+1E-16,df)` and `T.INV(0.5000001,df)`;
- `T.INV(0.46,df)`, `T.INV(0.47,df)`, `T.INV(0.53,df)` and `T.INV(0.54,df)` (c from 0.06 to 0.08);
- `T.INV.2T(0.92,df)` and `T.INV.2T(0.93,df)`.

## Acceptance criteria
- [ ] A probes-numeric case with the cells above, filled by the oracle, committed.
- [ ] statistics.md "Centre quantiles are accurate" cites the T.INV cells. It records whether Excel is the true quantile
      there (mpmath at the double p) and where the switch to the rounded-y form lies between c = 0.05 and 0.1.
- [ ] If Excel disagrees with the engine, a functions-stat task is filed with the failing cells (no engine change here).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a follow-up task does that if the evidence calls for it); df 1 (T-2858).
