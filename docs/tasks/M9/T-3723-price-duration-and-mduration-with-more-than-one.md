---
id: T-3723
title: "PRICE, DURATION and MDURATION with more than one coupon on T-2733's r4 fresh sweep: 65 cells within 15 digits (Phase 1b)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2972]
defer: phase-1b
---

## Goal
PRICE, DURATION and MDURATION of a bond with more than one coupon left give Excel's double on
T-2733's post-freeze r4 sweep (the case in the cell list below), after T-2972's sum-order work.

## Context
- Gap triage 2026-11-01h (`make eval` on main b3389661, excel-graph). Scan: `scripts/exit_owners.py`'s `owned` rule
  over every failing cell, against the task files of origin/main and every open task-filing branch. No open task names
  these cells. T-2972 has the same root cause but owns only the bond-extreme probe. probes-numeric is
  ungated, so these cells are in no baseline.
- 65 numeric cells, median 1 ulp, max 7 ulp. None is visibly off, so the task is `defer: phase-1b` (decision 0114).
  Each has COUPNUM > 1 (Excel's G column). The one-coupon cells of the same case are T-3722's. Examples, Excel vs ours:
  - Calc!B242 `DURATION(40604,44391,2.97622,7.05123,4,0)`: 0.2584858953686096 vs 0.25848589536860955. C242
    `MDURATION(...)`: 0.09355914060918454 vs 0.09355914060918452.
  - Calc!B57 `DURATION(40854,42985,0.000218573,4.65768,1,2)`: 2.9298486482520136 vs 2.929848648252013.
  - Calc!A131 `PRICE(38886,39404,0.0319477,24.0346,1385.37,1,1)`: 13.353695486589524 vs 13.353695486589512 (the
    largest PRICE miss). A52 `PRICE(43316,43934,0.227738,2.62924,3055.87,1,2)`: 352.56247896720953 vs
    352.5624789672097.
- Six DURATION/MDURATION cells here were bit-exact on main before T-2733 and moved by 1 or 2 ulp with its `k + DSC/E`
  exponent order: B109, C109, B260, C260, B277, C277 (T-2733 Outcome).
- Order of work: T-2972 fits the Macaulay sum order, its division by the price, and MDURATION's division by `1 + yld/f`
  on the bond-extreme probe. This sweep is out-of-sample evidence for that model. Start by re-measuring the owned
  cells on T-2972's main, then rank further candidates only on what is left. The 4 PRICE cells share the coupon-sum
  order (financial.md "PRICE, DURATION and MDURATION sum order at a huge coupon rate and yield": the offline fit
  ranked the candidate orders on the bond-extreme probe only).
- Code: `CouponSchedule::price` and `duration_of` in crates/rustsheet-engine/src/functions/financial/financial_bonds.rs.
  Scripts: docs/research/t2733/ (`gen_fresh.py`, `r4.py`).
- Cells owned:
  - probes-numeric/t2733-price-r4-fresh.case.json (65): Calc!A52, A119, A131, A133, B27, B57, B89, B104, B105, B106, B109, B116, B123, B125, B141, B161, B167, B170, B172, B188, B197, B200, B203, B206, B216, B242, B253, B260, B267, B274, B277, B278, B282, B283, B286, B295, C27, C57, C89, C106, C109, C116, C125, C141, C161, C167, C170, C172, C188, C197, C200, C203, C206, C216, C242, C253, C260, C267, C274, C277, C278, C282, C283, C286, C295

## Acceptance criteria
- [ ] More of the 65 owned cells are bit-exact (`make eval-show`, before and after counts in the Outcome). No passing
      cell of either T-2733 fresh sweep, of the bond-extreme probe (T-2972's case) or of gen-m9
      starts failing. `make eval-baseline` records no new known-failure line in any eval/baseline/<suite>.txt. Or the
      candidates scored are recorded in financial.md (time-boxed, decision 0056).
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), each shown in the Outcome:
  - [ ] Hold-out counts: train and hold-out side by side (bit-exact / within 4 ulp / within 15 digits) for the chosen
        order and the correctly rounded one. The split and the margin are fixed before scoring.
  - [ ] Fresh sweep: this case stops being fresh once it is used to choose, so a new probes-numeric sweep is drawn
        after the freeze and oracle-filled, with its counts.
  - [ ] Gate A envelope result on that sweep, with every point outside listed.
  - [ ] `FITTED(T-3723, n_cells, range)` on any scored constant or cut, applied only inside its range. A structural
        choice of order (kind (c)) has no tag, but the Outcome says it is kind (c).
  - [ ] Reproducibility floor: no step fits differences below 4 ulp web against desktop
        (docs/research/oracle-calibration.md).
  - [ ] No keep-matched branch: no special case whose domain is defined by the cells that already match (decisions
        0056, 0082).
- [ ] `make check` green; reviewers approve.

## Out of scope
- One coupon left (T-3722). The bond-extreme probe's cells (T-2972, T-2973). YIELD (T-1592).

## Deferred (decision 0114)
`defer: phase-1b`: every cell this task owns is already within 15 significant digits of Excel, so it waits until
Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells
visibly off (then give it a `campaign:` stage instead).
