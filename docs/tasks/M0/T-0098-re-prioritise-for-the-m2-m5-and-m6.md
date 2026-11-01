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
Assessment of 2026-10-28: a full `rustsheet-eval run --desktop-owned` on main 8d2f9394, binary built from that commit.
Phase 1 is 418 of 506; 88 functions are below the rate, 39 of them 1 to 5 cells short (decision 0118's rate: a
function's `rate_visible` cells less floor(`rate_cells` / 1000)). M2's, M5's and M6's exits miss only on their probe
rows (`milestone_exit`: coverage, generated suite and ironcalc-calc rows met), with 2, 27 and 5 visibly-off cells:
- M2 (probe row M0-M2): probes/t2772-unary-plus-in-range-rows S!J2, S!J5, owner T-2885.
- M5: probes/t2831-formulatext-indirect-multi-cell (21 cells) T-2992; probes/text-255-units-from-cell (2),
  -array-element (1) and -rept-error-count (2) T-2627; probes/text-format-digits-date-token Sheet1!C25 T-2388.
- M6: probes/t2065-lift-averagea, -maxa, -stdevs, -sumsq and -vars Sheet1!E1, owner T-2824.

## Acceptance criteria
- [x] The exit owners not already being worked are P0: T-2885 (M2), T-2388 (M5), T-2824 (M6). T-2992 and T-2627 own
      the other 26 M5 cells and are being worked on branches with commits of 2026-10-28; their priority is left as
      it is on main, since an edit there would conflict with their claim commits and would not change who works them.
- [x] The owners of the 39 nearest functions' cells are P1 where they were lower: T-2611 (FVSCHEDULE), T-2791 (SKEW),
      T-2633 (BETADIST), T-2811 (BINOM.INV, GAMMADIST), T-2243 and T-2951 (FV, NPER, PV), T-2646 (VALUE, IMABS), T-2690
      (LINEST, LOGEST, GROWTH), T-2521 (CUMPRINC), T-1749 (DEVSQ, KURT), T-2561 (GAMMAINV), T-2982 (GAMMADIST), T-2217
      (RRI), T-0975 (LOGNORM.DIST), T-2636 (SEARCH's corpus-bench/53062 case, SMALL's gen-m9 A363 and FREQUENCY), and
      T-2906, a probe of the calipers formula_stress_test NORMINV and NORM.INV cells (it settles Excel's value; a fix
      task follows from it).
- [x] T-2855 is P1 for M8's probe row: it owns probes/table-edits/plain-range-delete-row-first, table-delete-column-qty
      and table-delete-row-all-data-below, three of the six M8 cases with visibly-off cells.
- [x] The cells of those functions no open task names are T-3021's (P1).
- [x] Last-digit work waits on the kernel it would redo: T-2146 depends on T-2510 (bgrat for BINOM/NEGBINOM), T-0967
      on T-2511 (T.INV on the ported incomplete beta). The kernel chain T-2507 to T-2511 was already ordered.

## Outcome
Done as above. Not changed: the database functions the 2026-10-27 dashboard listed (DCOUNTA, DMAX, DVAR, DPRODUCT)
and NUMBERVALUE's and ARRAYTOTEXT's cells pass on main 8d2f9394 (T-2441 and T-2250 fixed them); tasks being worked on
branches were not edited.

## Out of scope
- Demotions (P1 tasks deferred to phase-1b, Phase 3 API tasks) and the merge-train order.
