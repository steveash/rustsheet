---
id: T-0098
title: "Re-prioritise for the M2, M5 and M6 exits and the cheapest Phase 1 lifts (2026-10-28 assessment)"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Priorities steer agents to the work that moves the project's measures most: the M2, M5 and M6 exits and the
functions nearest the Phase 1 rate (decision 0118).

## Context
Assessment of 2026-10-28 on main, from a full `rustsheet-eval run --desktop-owned` (Phase 1 412 of 506; 94 below the
rate, 42 of them 1 to 5 cells short) and the probe-milestone rows (M2's, M5's and M6's exits miss only on their
probes: 2, 28 and 5 visibly-off cells).

## Acceptance criteria
- [x] The milestone-exit owners are P0: T-2885 (M2, 2 cells), T-2824 (M6, 5), T-2388 (M5, 1). T-2992 (M5, 21) and
      T-2627 (M5, 5) are in progress on their branches and were left as they are; T-2441's NUMBERVALUE cell is in T-3017.
- [x] The owners of the cheapest functions are P1: T-2611 (FVSCHEDULE), T-2791 (SKEW), T-2633 (BETADIST), T-2811
      (BINOM.INV, GAMMADIST), T-2243 and T-2951 (FV, NPER, PV), T-2646 (VALUE, IMABS), T-2690 (LINEST, LOGEST, GROWTH),
      T-2521 (CUMPRINC), T-1749 (DEVSQ, KURT), T-2561 (GAMMAINV), T-2982 (GAMMADIST), T-2217 (RRI), T-2906 (NORMINV,
      NORM.INV), T-0975 (LOGNORM.DIST); and the open semantics tasks T-2855 and T-2636 (SMALL, SEARCH, FREQUENCY).
- [x] The unowned cells of those functions are listed for an owner in T-3017 (P1).
- [x] Last-digit work waits on the kernel it would redo: T-2146 depends on T-2510 (bgrat for BINOM/NEGBINOM), T-0967
      on T-2511 (T.INV on the ported incomplete beta). The kernel chain T-2507 to T-2511 was already ordered.

## Outcome
Done as above. Not changed: the database functions the 10-27 dashboard listed (DCOUNTA, DMAX, DVAR, DPRODUCT) have no
visibly-off cell on main any more; tasks being worked on branches were not edited (a priority edit on main would
conflict with their claim commits).

## Out of scope
- Demotions (P1 tasks deferred to phase-1b, Phase 3 API tasks) and the merge-train order: not part of this change.
