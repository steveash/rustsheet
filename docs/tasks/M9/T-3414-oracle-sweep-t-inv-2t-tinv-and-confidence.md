---
id: T-3414
title: "Oracle sweep: T.INV.2T, TINV and CONFIDENCE.T at p near 1 for df 2 to 1E10 (Excel's centre digits), with T.DIST.2T at Excel's results"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
defer: phase-1b
---

## Goal
A probes-numeric case, filled by the oracle, gives T-3415 enough Excel points to fit (and hold out) Excel's two-tailed
Student-t quantile at the centre (p just below or above 1, df from 2 up), where Excel is 1E-15 to 4E-15 off the true
quantile and the engine is on it.

## Context
- Gap triage 2026-10-31 (`make eval` on main fbcd47e96). On T-3349's fresh sweep, 21 visibly-off cells of
  probes-numeric/t3349-t-inv-2t-sweep with p from 0.99943 to 1.0000000006 and df from 2 to 5.7E9 have Excel 0.8E-15 to
  3.7E-15 relative off the true quantile (mpmath 1.4.1 at 60+ digits, at the double p) and ours within 4E-16 of it.
  Example: A187 `T.INV.2T(0.999999999999996,402)` Excel 5.0123657350064685E-15, ours 5.0123657350064874E-15 (true value
  within 5E-17 of ours). T-3415 owns the cells and lists them.
- These 21 sweep points (22 cells with probes/t3349-t-inv-2t-near-one A5, as T-3415 counts them) over df 2 to 5.7E9 and c = 1 - p from 6E-4 down to 4E-15 are too few to fit a form with a hold-out split
  (decision 0188 R4.1), so the fit is not attempted blind (gap-triager rule; T-0942 is the pattern).
- docs/excel-semantics/statistics.md "Two-tailed Student-t quantile with p near 1 is accurate in Excel (T-3349)" has
  the rule and its sweep note.
- Phase 1b (decision 0188 R5): the engine is on the math at every one of these points, so they are inside Gate A and
  the only consumer, T-3415, is deferred. This sweep waits with it.

## Acceptance criteria
- [ ] A probes-numeric case (the `excel-probe` skill; excel-graph), seeded and committed with its oracle sidecar, with
      at least 600 cells: `T.INV.2T(p,df)` for c = 1 - p log-uniform in [1E-15, 1E-3] (and 120 cells with p = 1 + c,
      c in [1E-15, 1E-9]), df log-uniform in [2, 1E10] with 60 cells at each of df 2, 3, 5, 10, 30; `TINV` at 60 of
      the same points; `CONFIDENCE.T(p,1,n)` at 60 points with n = df + 1; and `T.DIST.2T(x,df)` at Excel's own
      `T.INV.2T` result in the next column of the first 120 rows (a cell formula referencing it).
- [ ] Each p and df is written as a literal that round-trips to the intended double (17 significant digits), and the
      case notes the generator seed.
- [ ] A short scoring table in T-3415's Context (cells within 15 digits of the true quantile for Excel and for the engine,
      per df band), from a script committed under docs/research/t3414/ (mpmath, 60 digits).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change (T-3415).
- df 1 (T-2858's 1/TAN form) and tiny p (T-3386).
