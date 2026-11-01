---
id: T-3620
title: "YIELD one-step bonds: yield-one-step-start A162, B19 and B24 are 1 to 4 ulp off Excel after T-3215's start (Phase 1b)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The three YIELD cells of probes/yield-one-step-start that T-3215's start leaves 1 to 4 ulp off Excel take Excel's
double, or the Outcome records per cell which start or step form gives it and why the engine does not take it.

## Context
- Gap triage 2026-11-01 (`make eval` on agent/harness/T-3529-gate-a-chisq-gamma-beta e3be88d4a, origin/main 4e2f5f3df
  plus T-3529's harness commits; excel-graph evidence). `scripts/exit_owners.py`'s `owned` rule over every failing cell,
  against the task files of the working tree, origin/main and every unmerged triage branch: no open task owns these
  three cells. All are within 15 digits (`NumericUlp`), so `defer: phase-1b` (decision 0114, decision 0188 R5).
- The probe (T-2974 attempt 3): bonds whose Newton takes exactly one step from the start, so the value shows the
  start's last bit. T-3215 fitted the start's denominator `(100 T - q T/2) - q/4` in doubles (financial.md "YIELD stops
  short of PRICE's root", T-3215 paragraph: 369/370 lab cells bit-exact) and closed T-3152's A37; these three stayed
  in eval/baseline/probes.txt with no owner:
  - S!A162 `YIELD(DATE(2024,3,1),DATE(2027,6,30),0.1,95,99.976694,2,0)`: Excel 0.11842330011243947, ours
    0.11842330011243941 (4 ulp low; frequency 2, basis 0)
  - S!B19 `YIELD(DATE(2024,3,1),DATE(2053,10,1),0.06,112,105.779438,1,0)`: Excel 0.05272923315300489, ours
    0.052729233153004894 (1 ulp high; frequency 1)
  - S!B24 `YIELD(DATE(2024,3,1),DATE(2033,1,10),0.06,112,100.352852,1,0)`: Excel 0.04368660877095412, ours
    0.04368660877095414 (3 ulp high; frequency 1)
- Evidence is already cached: the probe's 360 cells and the fit lab (`uv run rustsheet-fitlab yield-caret`,
  kernels/yield_caret.py) with the 1,086 YIELD probe cells financial.md's T-3215 table scores. No sweep is needed before
  scoring candidates; R4's fresh sweep is still needed after the freeze if a fit lands.
- Split: T-2788 (research, YIELD's last ulps at the root) owns other yield-start-one-input, yield-start-rate-term and
  gen-m9 cells, not these; whichever lands a start or step change that fixes the other's cells records it in both.
- Code: `yield_start` and `excel_yield_newton` in
  crates/rustsheet-engine/src/functions/financial/financial_bonds/yield_solver.rs.
- Cells:
  - probes/yield-one-step-start.case.json (3): S!A162, B19, B24

## Acceptance criteria
- [ ] Each of the three cells is bit-exact with Excel, or the Outcome lists the start and step forms tried with their
      lab counts (financial.md, decision 0056).
- [ ] Their known-failure lines disappear from eval/baseline/probes.txt (`make eval-baseline`) with no new line in any
      baseline; `rustsheet-fitlab yield-caret`'s 369/370 basis-0 bit-exact count does not drop; no YIELD or ODDFYIELD
      cell moves from within 15 digits to visibly off.
- [ ] Any fit follows decision 0188 R4 (reviewer-excel-fidelity check 7), each guard shown in the Outcome: hold-out
      counts (train and hold-out side by side, bit-exact / 4 ulp / 15 digits, the chosen candidate against the
      correctly rounded one, margin stated before scoring); a fresh log-uniform sweep oracle-filled after the freeze;
      the Gate A envelope result on that sweep with every point outside listed; each fitted piece tagged
      `FITTED(T-3620, n_cells, range)` and applied only inside its range; no step fits differences below the
      reproducibility floor (4 ulp web against desktop, docs/research/oracle-calibration.md; B19 at 1 ulp is below it, so
      it is a target only if the same form fixes A162 or B24); no keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-2788's cells; ODDFYIELD (it shares the start, so its cells are a regression check only).
