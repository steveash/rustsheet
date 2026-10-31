---
id: T-3415
title: "T.INV.2T, TINV and CONFIDENCE.T centre at p near 1, df 2 up: Excel is 1E-15 to 4E-15 off the true quantile on 22 cells where ours is on it"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3414]
defer: phase-1b
---

## Goal
At the two-tailed centre (p just below or above 1, df from 2 up) T.INV.2T, TINV and CONFIDENCE.T give Excel's digits,
which are 1E-15 to 4E-15 relative off the true quantile, inside the measured range only, with accurate math kept
outside it; or the forms tried are recorded in docs/excel-semantics/statistics.md.

## Context
- Gap triage 2026-10-31 (`make eval` on main fbcd47e96). T-2724's centre crossing (`student_t_centre` in
  crates/rustsheet-engine/src/functions/statistical/t_dist.rs) is within 4E-16 of the true quantile at every cell
  below (mpmath 1.4.1 at 60+ digits, T.INV.2T at the double p). Excel is not: 0.8E-15 to 3.7E-15 relative off, so the
  cells are visibly off (more than half a unit in the 15th digit from Excel) though the engine is on the math.
- Needs an oracle sweep: 22 cells over df 2 to 5.7E9 and c = 1 - p from 6E-4 to 4E-15 do not support a fit with a
  hold-out split, so this task depends on T-3414's sweep and is not attempted blind.
- Examples (Excel, ours; true value within 5E-17 relative of ours unless noted):
  - probes-numeric/t3349-t-inv-2t-sweep A187 `T.INV.2T(0.999999999999996,402)`: 5.0123657350064685E-15,
    5.0123657350064874E-15 (Excel 3.7E-15 off).
  - A122 `T.INV.2T(0.999999999999998,2)`: 2.8261664256307887E-15, 2.826166425630795E-15 (Excel 2.2E-15 off).
  - A152 `T.INV.2T(1.00000000059497,17)` (the negative side): -7.567249884633081E-10, -7.567249884633087E-10.
  - A213 `T.INV.2T(0.999426378371611,9348283795)`: 7.189281582797262E-4, 7.189281582797248E-4 (Excel 1.6E-15 off).
  - probes/t3349-t-inv-2t-near-one A5 `CONFIDENCE.T(0.999999999989522,1,3)` (df 2): 8.555211852081089E-12,
    8.555211852081079E-12 (true 8.5552118520810793E-12; T-3349's Outcome classed it Phase 1b under decision 0188 R5).
- This contradicts no recorded probe: T-3349's 44 probe cells from df 2 up are within 15 digits of the true value, and
  the sweep shows Excel's centre error is larger at other df and c (statistics.md entry updated by this triage).
- Hypotheses: Excel's centre is the rounded-y form of T-3061 (y = I^-1 rounded to a double, then SQRT(df*(1/y-1))) with
  y near 1; or a series in c with Excel's own constants. Check T-3061's form here first: one kernel for both
  (directive 3).
- Phase 1b (decision 0188 R5): every cell is inside Gate A (the engine is within 1E-9 of the math), and the cells are a
  fresh-sweep and probe residue, so the task is deferred.
- Cells:
- probes-numeric/t3349-t-inv-2t-sweep.case.json (21): Calc!A122, A143, A146, A149, A152, A160, A173, A174, A178, A187, A191, A193, A204, A205, A212, A213, A218, A233, A234, A248, A250
- probes/t3349-t-inv-2t-near-one.case.json (1): Calc!A5

## Acceptance criteria
- [ ] The probes/t3349-t-inv-2t-near-one Calc!A5 known-failure line disappears from eval/baseline/probes.txt with no
      new known-failure line in any baseline; the 21 probes-numeric cells above pass (`make eval-show
      CASE=probes-numeric/t3349-t-inv-2t-sweep.case.json`), or the forms tried and their scores are in statistics.md.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7), recorded in the Outcome:
  - hold-out counts: the fit-lab split (about 30% by cell hash) over T-3414's sweep, train and test exact counts side
    by side; the form beats the correctly rounded candidate on test;
  - a fresh log-uniform sweep filled after the form is frozen, and its result;
  - the Gate A envelope result on that sweep (points within 15 digits of Excel, within 1E-9 of the math, outside both,
    error against number);
  - every fitted constant or cut-point tagged `/// FITTED(T-3415, <n> cells, <range>)`, applied only inside the
    measured df and c range, accurate math (T-2724's centre) outside it;
  - no fitting below the oracle's reproducibility floor (decision 0112: web against desktop up to 4 ulp): cells where
    Excel and the math differ by 4 ulp or less are not fit targets;
  - no branch whose only purpose is to keep an already-matching cell matching.
- [ ] `make check` green; reviewers approve.

## Out of scope
- df 1 (T-2858), tiny p (T-3386), the general region (T-2859, T-3061).
