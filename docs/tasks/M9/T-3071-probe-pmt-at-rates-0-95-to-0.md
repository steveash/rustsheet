---
id: T-3071
title: "Probe PMT at rates -0.95 to -0.5 with fv and type 1 (and each alone) so T-2951's rate -0.9 PMT bits can be fitted"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An oracle-filled probe case gives Excel's PMT bits at rates from -0.95 to -0.5 with an fv, with type 1, and with
both, so T-2951 can tell which of the two moves Excel's PMT 3 to 5 ulp below the exact payment at rate -0.9.

## Context
T-2951 (blocked, time-boxed) scored nine candidate PMT forms on PMT.xlsx (ironcalc-calc/FINANCIAL, Excel's cached
values) and m9-fin-pmt (`docs/research/t2951_pmt_order_score.py`; financial.md "Annuity evaluation order and
`(1+r)^n`"). None reproduces the three rate -0.9 cells with fv 300 and type 1 (`PMT(-0.9,3,120,300,1)` =
-2703.783783783783, `PMT(-0.9,3,100,300,1)` = -2703.603603603602, `PMT(-0.9,1,100,300,1)` = -3099.999999999999): every
form is 3 to 6 ulp below Excel's magnitude, and the miss does not depend on L = ln(1+rate) (a consistent L cancels at
n = 1). At rate -0.9 with fv 0 and type 0 (G10, G11, G18) the exact payment and the Excel-L/EXP form already agree
with Excel. PMT.xlsx has no rate -0.9 cell with fv alone or type 1 alone, and m9-fin-pmt has no rate below -0.02, so
the cause (fv, type 1, or the pair) cannot be separated from the cached values. PPMT's negative-rate probe
(probes-numeric/ppmt-negative-rate-sweep, T-2588) covers PPMT only.

## Acceptance criteria
- [ ] A spec case `probes-numeric/pmt-negative-rate-fv-type` with PMT at rates -0.95, -0.9, -0.8, -0.7, -0.6 and -0.5,
      nper 1, 2, 3, 5 and 10, pv 100 and 120, in four shapes per point: fv 0 type 0, fv 300 type 0, fv 0 type 1,
      fv 300 type 1 (plus `TRUE` for type in a few cells), about 200 cells; oracle-filled per the `excel-probe` skill,
      no pending cells.
- [ ] Each cell's formula is a literal call (no cell references), so `rustsheet-fitlab` and
      `docs/research/t2951_pmt_order_score.py` can read its arguments.
- [ ] financial.md "PMT, IPMT and PPMT are the exact payment" names the case and says, from the filled values, which
      shapes are off the exact payment (computed at high precision) and by how many ulp; T-2951's "Blocked because"
      says the sweep landed, so the task can be unblocked.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing PMT (T-2951 does that once the sweep lands).
- IPMT, PPMT, CUMIPMT and CUMPRINC.
