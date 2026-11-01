---
id: T-3722
title: "DURATION and MDURATION with one coupon left: 55 cells of T-2733's fresh sweeps, 1 to 2 ulp (Phase 1b)"
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
DURATION and MDURATION of a bond with one coupon left (COUPNUM = 1) give Excel's double on T-2733's two fresh
sweeps, not only its 15 digits.

## Context
- Gap triage 2026-11-01h (`make eval` on main b3389661, excel-graph). Scan: `scripts/exit_owners.py`'s `owned` rule
  over every failing cell, against the task files of origin/main and every open task-filing branch. T-2733 (done) drew
  its r4 and n1 R4.2 fresh sweeps (the probes-numeric cases in the cell list below). Its follow-ups own other
  cases: T-2972 owns the DURATION/MDURATION residue of the bond-extreme probe, T-2973 the 6 cells it moved.
  So no open task names the fresh sweeps' failing cells. probes-numeric is ungated, so these cells are in no baseline.
- All 55 are numeric, 1 or 2 ulp, and none is visibly off (`attributed_off` has no t2733 cell). Hence
  `defer: phase-1b` (decision 0114).
- Every one has COUPNUM = 1 (Excel's own G column in each case). The engine discounts the single cash flow at the
  exponent `DSC/E + N - 1` = `DSC/E` and divides `t·cf/v^t` by `cf/v^t`. With one cash flow Excel's quotient is not
  exactly t either, but it lands on the other side:
  - n1-fresh Calc!B106 `DURATION(38696,38758,232.89,0.013693,2,4)`: Excel 0.16666666666666663, ours
    0.16666666666666669. The double nearest 1/6 is 0.16666666666666666, so Excel is 1 ulp below it and we are 1 ulp
    above.
  - n1-fresh Calc!B196 `DURATION(41440,41511,904.715,6.35392,2,0)`: Excel 0.19444444444444445, ours
    0.19444444444444442.
  - r4-fresh Calc!B187 `DURATION(43699,43740,35.1601,0.000210518,1,1)`: Excel 0.11232876712328768, ours
    0.1123287671232877. C187 `MDURATION(...)`: Excel 0.11230512487300966, ours 0.11230512487300967.
- T-2733's Outcome: main before T-2733 matched 12 of these cells (n1-fresh B106, C106, B107, C107, B114, C114, B118,
  C118, B120, C120, B126, C126). They were lost to the `k + DSC/E` exponent order, which gained 44 + 44 cells on the
  same sweep. So the single-coupon DURATION likely has its own branch, the way Excel's single-coupon PRICE discounts
  linearly (financial.md "PRICE, DURATION and MDURATION sum order at a huge coupon rate and yield"). Candidates to rank:
  - `DSC/E/f` as a closed form (t only, no cash flows);
  - the linear discount `1 + DSC/E·yld/f` in place of Excel's `^`;
  - the time unit: t in periods divided by f at the end, against t in years;
  - MDURATION's division by `1 + yld/f`, against multiplication by its reciprocal.
- Code: `duration_of` in crates/rustsheet-engine/src/functions/financial/financial_bonds.rs. Generator and scorer:
  docs/research/t2733/ (`gen_fresh.py`, `r4.py`).
- Cells owned:
  - probes-numeric/t2733-price-n1-fresh.case.json (42): Calc!B106, B107, B110, B114, B116, B118, B120, B124, B126, B131, B132, B136, B142, B146, B157, B158, B160, B163, B169, B170, B196, C106, C107, C110, C114, C116, C118, C120, C124, C126, C131, C132, C136, C142, C146, C157, C158, C160, C163, C169, C170, C196
  - probes-numeric/t2733-price-r4-fresh.case.json (13): Calc!B187, B226, B243, B257, B273, B284, B293, C187, C226, C257, C273, C284, C293

## Acceptance criteria
- [ ] More of the 55 owned cells are bit-exact (`make eval-show` on both cases, before and after counts in the
      Outcome). No passing DURATION, MDURATION or PRICE cell of either sweep, of the bond-extreme probe (T-2972's case) or
      of gen-m9 starts failing. `make eval-baseline` records no new known-failure line in any eval/baseline/<suite>.txt.
      Or the candidates scored are recorded in financial.md (time-boxed, decision 0056).
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), each shown in the Outcome:
  - [ ] Hold-out counts: train and hold-out side by side (bit-exact / within 4 ulp / within 15 digits) for the chosen
        branch and the correctly rounded one. The split and the margin are fixed before scoring.
  - [ ] Fresh sweep: a new probes-numeric sweep of one-coupon bonds, drawn after the change is frozen and
        oracle-filled, with its counts.
  - [ ] Gate A envelope result on that sweep, with every point outside listed.
  - [ ] `FITTED(T-3722, n_cells, range)` on any scored constant or cut, applied only inside its range. A structural
        choice among candidates (kind (c)) has no tag, but the Outcome says it is kind (c).
  - [ ] Reproducibility floor: no step fits differences below 4 ulp web against desktop
        (docs/research/oracle-calibration.md).
  - [ ] No keep-matched branch: no special case whose domain is defined by the cells that already match (decisions
        0056, 0082).
- [ ] financial.md's "PRICE, DURATION and MDURATION sum order at a huge coupon rate and yield" entry records the
      single-coupon DURATION rule with cell citations.
- [ ] `make check` green; reviewers approve.

## Out of scope
- More than one coupon left (T-3723 on the r4 sweep, T-2972 on the bond-extreme probe). The single-coupon PRICE
  (bit-exact on both sweeps). YIELD (T-1592).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
