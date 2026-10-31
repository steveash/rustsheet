---
id: T-3224
title: "GAMMA.DIST, GAMMADIST and CHISQ.DIST cumulative last digits on the probes-numeric shape sweeps (43 cells no task names)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2638, T-2367]
defer: phase-1b
---

## Goal
The 43 cumulative incomplete-gamma cells listed below (GAMMA.DIST and GAMMADIST with cumulative TRUE, CHISQ.DIST
cumulative, CHISQ.DIST.RT), all already within 15 significant digits of Excel, match Excel to the bit, or the task
records the candidate formulations tried with their bit-exact counts (decision 0056 time box).

## Context
Split: T-3234 (deferred M9 fitting) also lists these cumulative cells; this task owns them.

Gap triage 2026-10-28 (`make eval` on 40844417, excel-graph evidence). An ownership check over every failing cell (not
only the visibly-off ones `make unowned-off` lists) found these cells named by no open task, here or on a pushed
`agent/*` branch. Neighbouring owners, none of which lists them: T-2695 (the same functions' gen-m9 sweep cells, deferred),
T-2367 (the incomplete-gamma ulp residue T-0971 moved; it names other cells of these probes), T-2638 (the visibly-off
tails and densities), T-2852 (the density, `gamma_density`: the cumulative FALSE cells of these probes are left to it and
are not owned here).

Every cell is 1 to 4 ulp off and none is visibly off, so this is Phase 1b work and stays deferred. It depends on T-2638
and T-2367 because their kernel changes move these cells: re-run the list after them before any fit.

Root cause (suspected): Excel's regularized lower incomplete gamma P(a, x) is not correctly rounded, and the engine's
accurate kernel (`crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs`,
`incomplete_gamma_pair`, T-0971) is not Excel's. A 50-digit mpmath reference (this triage) shows the error is Excel's
in some cells and the engine's in others, so this is fitting Excel's formulation (series and continued-fraction choice,
prefactor), not an accuracy fix. Ulps from the 50-digit value, Excel / ours:
- `GAMMA.DIST(50,50,1,TRUE)` (m9-gamma-dist-a50 A5): +0.28 / -0.72; `GAMMA.DIST(500,500,1,TRUE)` (m9-gamma-dist-a500
  A5): -0.44 / -3.44; `GAMMA.DIST(9,9,1,TRUE)` (m9-gamma-dist-a9 A5): -0.92 / -1.92; `GAMMA.DIST(1,2,1,TRUE)`
  (m9-gamma-dist-a2 A3): -2.45 / -1.45; `GAMMADIST(25,10,3,TRUE)` (t1048-gammadist A11): +4.01 / +3.01.
- CHISQ.DIST at a small x against the shape (t1049-chisqdist): `CHISQ.DIST(0.5,20,TRUE)` A15 -19.21 / -0.21,
  `CHISQ.DIST(0.1,20,TRUE)` A7 -6.51 / -2.51, `CHISQ.DIST(0.1,5,TRUE)` A5 +2.86 / -0.14. Excel loses up to 19 ulp where
  x/2 is far below a = k/2, which points at its lower series (its term count or summation order), and at x near a the
  two are about a unit apart either way.
- Equal arguments after scaling give equal cells (`GAMMA.DIST(50,50,1,TRUE)`, `(100,50,2,TRUE)`,
  `(50000000,50,1000000,TRUE)` are all 0.5188083154720433 in Excel), so the miss is in P(a, x/beta), not in the
  scaling.

Examples (`make eval-show CASE=probes-numeric/<case>.case.json`), Excel vs ours:
- m9-gamma-dist-a500 Calc!A5 `=GAMMA.DIST(500,500,1,TRUE)`: 0.5059471461707603 vs 0.50594714617076 (3 ulp)
- t1048-gammadist Calc!A5 `=GAMMADIST(3,2,1.5,TRUE)`: 0.5939941502901619 vs 0.5939941502901618 (1 ulp)
- t1049-chisqdist Calc!A15 `=CHISQ.DIST(0.5,20,TRUE)`: 2.0942485399973563E-13 vs 2.094248539997361E-13
- ironcalc-calc STATISTICAL/CHISQ.xlsx Sheet1!G2 `=CHISQ.DIST.RT(A2,B2)`: 0.015254394655769613 vs
  0.015254394655769615; Sheet1!E4 `=CHISQ.DIST(A4,B4,C4)`: 0.030107210682426052 vs 0.030107210682426062.

Owned cells:
- ironcalc-calc/STATISTICAL/CHISQ.xlsx (2): Sheet1!E4, Sheet1!G2
- probes-numeric/m9-gamma-dist-a2.case.json (4): Calc!A3, A13, A23, A25
- probes-numeric/m9-gamma-dist-a9.case.json (6): Calc!A5, A7, A15, A17, A25, A27
- probes-numeric/m9-gamma-dist-a50.case.json (3): Calc!A5, A15, A25
- probes-numeric/m9-gamma-dist-a500.case.json (3): Calc!A5, A15, A25
- probes-numeric/t1048-gammadist.case.json (14): Calc!A5, B5, A11, B11, A13, B13, A21, B21, A22, B22, A25, B25, A26, B26
- probes-numeric/t1049-chisqdist.case.json (11): Calc!A5, A7, A15, A29, A31, A37, A45, A55, A58, A59, A63

Evidence is cached in `eval/` (excel-graph). If these 43 points and T-2695's 86 gen-m9 cells cannot separate the
candidate formulations, file an oracle sweep (research lane, `excel-probe` skill) of GAMMA.DIST cumulative over a dense
(a, x) grid around x = a and at x far below a, and depend on it, rather than fitting blind.

## Acceptance criteria
- [ ] The owned cells match Excel bit for bit in `make eval`, or the Outcome lists each candidate formulation with its
      bit-exact count over the owned cells and T-2695's cells (decision 0056).
- [ ] The CHISQ.xlsx Sheet1!E4 and Sheet1!G2 lines leave eval/baseline/ironcalc-calc.txt for each one fixed, and no new
      known-failure lines appear in any `eval/baseline/` file (probes-numeric is ungated, so its cells are checked
      with `make eval-show`).
- [ ] docs/excel-semantics/statistics.md records the finding, citing t1049-chisqdist A15 and A7 (Excel's cdf up to
      19 ulp low where x is far below the shape).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The density cells (cumulative FALSE) of these probes: T-2852.
- The gen-m9 cells: T-2695. The quantiles (GAMMA.INV, CHISQ.INV): T-0972, T-2982 and their follow-ups.

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
