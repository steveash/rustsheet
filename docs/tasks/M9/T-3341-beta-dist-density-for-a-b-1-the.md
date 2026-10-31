---
id: T-3341
title: "BETA.DIST density for a, b > 1: the Loader port's bit-level residue on t2464-r2-density-large-p1 and t2513-r3-loader-bd0-b (13,034 cells within 15 digits)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2586, T-3220]
defer: phase-1b
---

## Goal
BETA.DIST's density for both shapes above 1 matches Excel to the bit on the two Loader sweeps below, where it is already
within 15 significant digits everywhere.

## Context
Gap triage 2026-10-31 (`make eval`, excel-graph evidence). T-2507 (done) ported Excel's Loader density
(`loader.rs` `excel_beta_density`, `excel_bd0`) and brought t2464-r2-density-large-p1 within 15 digits on all 13,824
cells (3,874 bit-exact then). Its follow-up T-3220 owns only the 661 cells the port left visibly off, and those are in
other cases. No open task names these two cases, so their bit-level residue had no owner. None of these cells is
visibly off (decision 0083), so this is Phase 1b work (decision 0114) and stays deferred.

Failing cells on this run (ulps off Excel):
- probes-numeric/t2464-r2-density-large-p1.case.json: 9,950 of 13,824 cells (1 ulp 3,869; 2 to 4 ulp 4,681; 5 to 16
  ulp 1,400). Runs at a, b > 1 across the bd0 cut, the a0 = 8 straddle, the shape constants and a hold-out. Example:
  Calc!A1 `BETA.DIST(0.225000137,2.5,3.5,FALSE)` Excel 1.5328408862786385, ours 1.5328408862786378.
- probes-numeric/t2513-r3-loader-bd0-b.case.json: 3,084 of 4,992 cells (1 ulp 2,167; 2 to 4 ulp 844; 5 to 16 ulp 73).
  Runs at Loader bd0 ratios -0.3, -0.15, 0.15 and 0.3 for ten shape pairs from (2, 5) to (10, 10). Example: Calc!A1
  `BETA.DIST((7789991*2^-23+8121047*2^-49),3.0,3.0,FALSE)` Excel 0.13174480397454205, ours 0.13174480397454208.
This task owns every failing cell of both cases (the run's list, `make eval-show CASE=probes-numeric/<case>`); the
cases are ungated, so there are no baseline lines.

Open pieces that move these bits: Loader's constant and the bd0 series (T-2586, research, in progress; its r3 libraries
under docs/research/t2464/r3-*), and the visibly-off residue of the same kernel (T-3220). Re-run the counts after both land
before any fit; a fit made first would be overwritten. Semantics: docs/excel-semantics/statistics.md, the beta
density entries of T-2503, T-2507 and T-2513.

## Acceptance criteria
- [ ] Every failing cell of the two cases above is bit-exact, or the Outcome lists the ones left with their ulps after
      decision 0056's two attempts, with no new known-failure line in any eval/baseline file (`make eval-baseline`) and
      no cell newly visibly off.
- [ ] The forms tried, with their bit-exact counts on both cases, are in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 661 visibly-off Loader cells and BETADIST_BETAINV.xlsx G3 (T-3220); min(a, b) <= 1 (T-2821, T-3035); both shapes
  large (T-3244); the large-exponent case t2464-r2-log1p-hix (T-3250).
