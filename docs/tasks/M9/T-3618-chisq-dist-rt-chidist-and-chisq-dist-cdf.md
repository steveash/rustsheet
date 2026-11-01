---
id: T-3618
title: "CHISQ.DIST.RT, CHIDIST and CHISQ.DIST cdf last digits on m9-chisq-dist-rt, m9-legacy-chidist, t1049-chisqdistrt and m9-chisq-dist: 29 cells 1 to 6 ulp off Excel that no task names (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3622]
defer: phase-1b
---

## Goal
The 29 chi-squared cdf cells listed below, all already within half a unit of Excel's 15th significant digit, take
Excel's double, or the Outcome records per cell why not (Excel, ours and the 50-digit value).

## Context
- Gap triage 2026-11-01 (`make eval` on agent/harness/T-3529-gate-a-chisq-gamma-beta e3be88d4a, which is origin/main
  4e2f5f3df plus T-3529's harness commits; excel-graph evidence). `scripts/exit_owners.py`'s `owned` rule over every
  failing cell, against the task files of the working tree, origin/main and every unmerged triage branch: these 29
  cells have no open owner. None is visibly off (verdicts: 28 `NumericUlp`, 1 `NumericClose`), so `defer: phase-1b`
  (decision 0114, decision 0188 R5).
- Each cell's only non-glue call is CHISQ.DIST.RT, CHIDIST or CHISQ.DIST, so the kernel is the regularized incomplete
  gamma Q(df/2, x/2) or P(df/2, x/2) (`crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/`, the
  half-integer path in `incomplete_gamma/half_integer.rs`, callers in `chisq.rs`). No case here has a math sidecar;
  the 50-digit values below are mpmath `gammainc(..., regularized=True)` at 50 digits, ulps of that value rounded to
  a double (positive = above). Excel is 0 to 4 ulp off the math, except A30 (20) and A34 (8) of m9-chisq-dist-rt:
  | cell | formula | Excel | ours | Excel vs math | ours vs math |
  |---|---|---|---|---|---|
  | m9-chisq-dist-rt A12 | `CHISQ.DIST.RT(1.5,3)` | 0.6822703303362125 | 0.6822703303362126 | 0 | +1 |
  | m9-chisq-dist-rt A13 | `CHISQ.DIST.RT(3,3)` | 0.39162517627108884 | 0.39162517627108917 | -2 | +4 |
  | m9-chisq-dist-rt A15 | `CHISQ.DIST.RT(12,3)` | 0.007383160505359771 | 0.007383160505359769 | +2 | 0 |
  | m9-chisq-dist-rt A20 | `CHISQ.DIST.RT(20,5)` | 0.0012497305630313747 | 0.0012497305630313753 | -3 | 0 |
  | m9-chisq-dist-rt A24 | `CHISQ.DIST.RT(20,10)` | 0.029252688076961065 | 0.02925268807696106 | -2 | -3 |
  | m9-chisq-dist-rt A25 | `CHISQ.DIST.RT(40,10)` | 1.6944743930067382E-05 | 1.6944743930067375E-05 | -1 | -3 |
  | m9-chisq-dist-rt A30 | `CHISQ.DIST.RT(200,50)` | 7.857610724654719E-20 | 7.85761072465472E-20 | -20 | -19 |
  | m9-chisq-dist-rt A34 | `CHISQ.DIST.RT(200,100)` | 1.1784500720979435E-08 | 1.1784500720979433E-08 | +8 | +7 |
  | m9-legacy-chidist A53, A54; t1049-chisqdistrt A23 | `CHIDIST(50,5)`, `CHISQ.DIST.RT(50,5)` | 1.3857973367009602E-09 | 1.3857973367009593E-09 | +4 | 0 |
  | m9-legacy-chidist A55, A56 | `CHIDIST(50,30)`, `CHISQ.DIST.RT(50,30)` | 0.01240206071890058 | 0.012402060718900584 | 0 | +3 |
  | m9-legacy-chidist A21, A22; t1049-chisqdistrt A3 | `CHISQ.DIST.RT(0.1,5)` | 0.9998376833880773 | 0.9998376833880774 | -1 | 0 |
  | m9-chisq-dist A63 | `CHISQ.DIST(50,100,TRUE)` | 6.953305247616096E-06 | 6.953305247616095E-06 | -3 | -4 |
  So it is neither side's accuracy: at some cells Excel is correctly rounded and ours is not (A13, A55/A56), at others
  the reverse (A15, A20, A53/A54). Matching it means reproducing Excel's formulation: a fit.
- Neighbouring tasks, none of which lists these cells (no cell is shared, so no split line is needed):
  - T-3461 (deferred) owns Excel's Q at half-integer shapes from 3/2 to 20 with a + 1 <= x <= 30 on its listed gen-m9
    and t3284 cells, with the t3285 half-integer sweep as evidence. CHISQ.DIST.RT(12,3) = Q(1.5, 6), (20,5) =
    Q(2.5, 10) and (50,5) = Q(2.5, 25) fall in its range; whichever task lands Excel's form first takes them and says so
    in both Outcomes.
  - T-2367 (deferred) owns m9-chisq-dist-rt A14 and the 1-ulp cells T-0971's kernel moved; T-3234 (deferred) owns
    other m9-chisq-dist and m9-gamma-dist cells; T-2695 (deferred) the gen-m9 CHIDIST sweep cells; T-3269 the deep upper
    tail past x = 30.
- Semantics: docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel (T-0971)" (forms already ruled
  out) and "Incomplete gamma sweep (T-1045)".
- Needs an oracle sweep: the cached evidence at integer shapes (even df) is these m9 probes' handful of points, too few
  for a train set and a hold-out (decision 0188 R4.1). T-3622 is the sweep this task depends on.
- Cells:
  - probes-numeric/m9-chisq-dist-rt.case.json (11): Calc!A12, A13, A15, A20, A24, A25, A27, A29, A30, A33, A34
  - probes-numeric/m9-legacy-chidist.case.json (12): Calc!A21, A22, A29, A30, A37, A38, A53, A54, A55, A56, A63, A64
  - probes-numeric/t1049-chisqdistrt.case.json (5): Calc!A3, A11, A15, A23, A24
  - probes-numeric/m9-chisq-dist.case.json (1): Calc!A63

## Acceptance criteria
- [ ] Each of the 29 listed cells is bit-exact with Excel, or within 1 ulp with the Outcome naming it, or the Outcome
      lists it with Excel, ours and the 50-digit value and the forms tried (recorded in statistics.md, decision 0056).
- [ ] probes-numeric is ungated (no baseline file), so the Outcome shows `make eval-show` (or a main-vs-HEAD diff of
      the report) for the 29 cells; `make eval-baseline` adds no line to any eval/baseline/ file, and no cell of any
      suite moves from within 15 digits to visibly off.
- [ ] Any fit follows decision 0188 R4 (reviewer-excel-fidelity check 7), each guard shown in the Outcome: hold-out
      counts (train and hold-out side by side, bit-exact / 4 ulp / 15 digits, the chosen candidate against the
      correctly rounded one, margin stated before scoring); a fresh log-uniform sweep oracle-filled after the freeze;
      the Gate A envelope result on that sweep with every point outside listed; each fitted piece tagged
      `FITTED(T-3618, n_cells, range)` and applied only inside its range; no step fits differences below the
      reproducibility floor (4 ulp web against desktop, docs/research/oracle-calibration.md); no keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cells T-3461, T-2367, T-3234, T-2695 and T-3269 list.
- The density (CHISQ.DIST with FALSE): T-3303 and T-3234.
