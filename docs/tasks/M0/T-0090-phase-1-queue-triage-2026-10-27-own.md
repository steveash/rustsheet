---
id: T-0090
title: "Phase 1 queue triage 2026-10-27: own, route and stage every function below the Phase 1 rate"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The task queue picks the work that moves the Phase 1 exit (decision 0114: every in-scope function at ≥ 99.9% of its
attributed Excel cells within 15 digits, every remaining failure filed, every Phase 1 milestone exit met): every
function below the rate has its visibly-off cells owned by tasks the queue will pick, the tasks that alone lift whole
functions carry a campaign stage, parked Phase 1b tasks hold no visibly-off cell, and the missing exit plumbing is
filed.

## Context
- Owner request (2026-10-27): find the most important items for exiting Phase 1 that are not soon in the queue, then
  address all six findings.
- Eval of 2026-10-27 at `main` 7f6b373a (calipers, poi and corpus-bench not fetched): Phase 1 not met, 393/506
  functions at the rate; 113 below need 101,563 cells fixed. `make ready-tasks` for both tiers listed mostly perf,
  lookup-probe and quality tasks, because every campaign task's lane (functions-stat, functions-financial) was busy
  (one task in flight per lane).
- Method: for each visibly-off Excel cell (closeness.rs rule: beyond 4 ulp and beyond half a unit in the 15th
  digit, or a non-numeric verdict), the attributed function (decision 0118 `sole_call`) and its owners under
  scripts/exit_owners.py `owned`, split into live, blocked, deferred-only and none.

## Acceptance criteria
- [x] Item 1 (incomplete-beta critical path): T-2508, T-2510, T-2511 raised to P1; T-2981 owns and routes the 70
      sweep cases (88,502 visibly-off cells) no kernel task named; T-2821 (bit-level, depends on parked T-2122 and
      T-2512) parked with `defer: phase-1b`.
- [x] Item 2 (single-task function lifts): campaign stage and priority on T-2617, T-2566, T-2641, T-2481, T-2665,
      T-2521, T-2094, T-2627, T-2811, T-2848, T-2855, T-2791, T-2613, T-2636, T-2441; priority raised on T-2146,
      T-2185, T-0975. T-2779, T-2841, T-2339, T-2834 and T-2572 were finished, and T-2548 blocked, on `main` before
      this landed (the T-0087 sweep re-checks their cells).
- [x] Item 3 (parked tasks owning visibly-off cells): T-2243 un-parked (`campaign: visible`); the visibly-off
      cells of the parked T-2730, T-2221, T-0968, T-2695, T-2367 and T-2765 handed to live tasks (T-2243, T-2611,
      T-2981, T-0925, T-2511, T-0967, T-2131, T-2636; the F.TEST/FTEST cells meant for T-2681, finished meanwhile, go to the
      T-0087 sweep), each parked task noting it.
- [x] Item 4 (desktop-only cells): T-2425 owns all 302 previously unowned FORECAST.ETS cells; T-0085 filed so the
      Phase 1 line treats M14-owned cells as filed (decision 0114).
- [x] Item 5 (unowned cells): T-0086 (`exit_owners.py --all-off`) and T-0087 (gap-triager sweep) filed.
- [x] Item 6 (exit rows): T-0088 (decision on how M1, M2, M7-M11 exits are measured) and T-0089 (the rows) filed.
- [x] `python3 scripts/tasks.py validate` passes; no code, baseline or quality file changes.

## Out of scope
- Fixing any cell; changing any gate.

## Outcome
- Unowned visibly-off attributed cells of functions below the rate: about 81,200 before, 478 after (the rest of
  the gap is now owned by open, non-deferred tasks); unattributed unowned 668 to 365. FORECAST.ETS cells are owned
  by the blocked M14 task T-2425 until T-0085 lands.
- Functions that one open campaign task now lifts on its own: the six D* functions (T-2617), IMREAL, IMSEC, IMCSCH,
  IMAGINARY (T-2841, finished on `main` meanwhile), XNPV (T-2566), PRICE, MDURATION, DURATION (T-2733), LINEST, GROWTH, LOGEST (T-2690),
  HYPGEOMDIST, HYPGEOM.DIST (T-2481), VALUE, IMABS (T-2641), PV, NPER, FV (T-2243),
  NORM.S.INV (T-0925) and about twenty more with one task each.
- Follow-ups: T-2981, T-0085, T-0086, T-0087, T-0088, T-0089.
