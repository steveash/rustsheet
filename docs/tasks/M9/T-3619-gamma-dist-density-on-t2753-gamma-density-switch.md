---
id: T-3619
title: "GAMMA.DIST density on t2753-gamma-density-switch and m9-gamma-dist-a1 A28: 64 cells 1 to 7 ulp off Excel inside T-3303's ranges that no task names (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3303]
defer: phase-1b
---

## Goal
The 63 failing cells of probes/t2753-gamma-density-switch and probes-numeric/m9-gamma-dist-a1 A28, all already within
half a unit of Excel's 15th significant digit, take Excel's double once T-3303 has settled the density's form at y up to
1E-15 and in the 1 to 6 ulp residue, or the Outcome records per cell why not.

## Context
- Gap triage 2026-11-01 (`make eval` on agent/harness/T-3529-gate-a-chisq-gamma-beta e3be88d4a, origin/main 4e2f5f3df
  plus T-3529's harness commits; excel-graph evidence). `scripts/exit_owners.py`'s `owned` rule over every failing cell,
  against the task files of the working tree, origin/main and every unmerged triage branch: no open task owns these 64
  cells. None is visibly off (60 `NumericUlp`, 4 `NumericClose`), so `defer: phase-1b` (decision 0114, decision 0188
  R5).
- The switch case (T-2753) is GAMMA.DIST's density at x / beta from 1E-19 to 1E-11 and near 2.2E-16, shapes 1, 2 and
  0.5, beta = 1 in column A and x = 1 in column B. T-2852 found Excel's form there (statistics.md "GAMMA.DIST's density
  below shape 3 is a log-space form (T-2753, T-2852)") and left the residue to T-3303, whose Context names "the switch
  A39, A40, B39, B41 ... about 20 others" at y up to 1E-15 but lists no switch cell. Every cell below is in one of
  T-3303's ranges (y up to 1E-15, or 1E-15 < y <= 16 with shape at least 0.5), so its form choice moves them: hence the
  dependency, and a re-score of this list before any work here.
- Ours against Excel: 1 ulp at 43 cells, 2 to 3 ulp at 15, 4 to 7 ulp at 6 (A5 4, B5 5, A19 6, B19 7, A33 4, B33 5).
  Against 50-digit mpmath (x^(a-1) e^(-x/b) / (Gamma(a) b^a)), Excel itself is 2 to 57 ulp off at y
  below 1E-11 (A18 `GAMMA.DIST(1E-16,2,1,FALSE)`: Excel 9.999999999999928E-17, -57 ulp; ours -55), and ours tracks it
  within 1 to 7 ulp. So this is reproducing Excel's formulation, never the math (Excel is always right). Examples:
  - S!A5 `GAMMA.DIST(1E-15,1,1,FALSE)`: Excel 1.0000000000000002, ours 1.000000000000001 (4 ulp)
  - S!B19 `GAMMA.DIST(1,2,1/1E-15,FALSE)`: Excel 1.0000000000000011E-30, ours 1.0000000000000024E-30 (7 ulp)
  - S!A3 `GAMMA.DIST(1E-17,1,1,FALSE)`: Excel 0.999999999999999, ours 0.9999999999999989 (1 ulp; the math rounds to 1)
  - m9-gamma-dist-a1 Calc!A28 `GAMMA.DIST(1500000,1,1000000,FALSE)`: Excel 2.2313016014842983E-07 (correctly rounded),
    ours 2.231301601484298E-07 (1 ulp); T-2852's Outcome lists it as lost bit-exact under its form.
- Split: T-3303 owns the density's form below shape 3 and the cells it lists; T-3234 owns probes/t2753-gamma-density-
  ratio-sweep and m9-gamma-dist-a1 A10, A20 and A30. No cell here is listed by either; whichever task lands a form that
  fixes a cell of the other records it in both Outcomes.
- Code: `log_form_density` and the density dispatch in crates/rustsheet-engine/src/functions/statistical/gamma.rs.
- Cells:
  - probes/t2753-gamma-density-switch.case.json (63): S!A3, A4, A5, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A23, A24, A25, A26, A27, A28, A29, A30, A31, A32, A33, A34, A37, A38, A41, A42, B4, B5, B10, B11, B12, B13, B14, B15, B17, B18, B19, B20, B21, B22, B23, B24, B25, B26, B27, B28, B29, B30, B31, B32, B33, B35, B36, B38, B40, B42
  - probes-numeric/m9-gamma-dist-a1.case.json (1): Calc!A28

## Acceptance criteria
- [ ] After T-3303 lands, each listed cell is bit-exact with Excel or within 1 ulp with the Outcome naming it, or the
      Outcome lists it with Excel, ours and the 50-digit value and the forms tried (statistics.md, decision 0056).
- [ ] The fixed cells' known-failure lines disappear from eval/baseline/probes.txt (`make eval-baseline`) with no new
      line in any baseline; no cell of any suite moves from within 15 digits to visibly off.
- [ ] Any fit follows decision 0188 R4 (reviewer-excel-fidelity check 7), each guard shown in the Outcome: hold-out
      counts (train and hold-out side by side, bit-exact / 4 ulp / 15 digits, the chosen candidate against the
      correctly rounded one, margin stated before scoring); a fresh log-uniform sweep oracle-filled after the freeze
      (T-3303's sweep counts only if filled after this task's freeze); the Gate A envelope result on that sweep with
      every point outside listed; each fitted piece tagged `FITTED(T-3619, n_cells, range)` and applied only inside its
      range; no step fits differences below the reproducibility floor (4 ulp web against desktop,
      docs/research/oracle-calibration.md); no keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cells T-3303, T-3234 and T-3219 (shape 50) list; the cdf.
