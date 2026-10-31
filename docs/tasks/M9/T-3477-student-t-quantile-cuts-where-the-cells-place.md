---
id: T-3477
title: "Student-t quantile cuts where the cells place them: T_CENTRE_C and T_TAIL_P (R4 guards)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
The Student-t quantile cuts `T_CENTRE_C` and `T_TAIL_P` in crates/rustsheet-engine/src/functions/statistical/t_dist.rs
sit where Excel's cells place them, each chosen under decision 0188 R4's guards, with its `FITTED` tag updated.

## Context
T-3395 moved each cut in full eval runs on code after T-2511 (a temporary env override, reverted; suites
probes-numeric, probes, gen-m9, ironcalc-calc) and scored every changed cell by its ulp distance to Excel (counts are
in the doc comments):
- `T_CENTRE_C` (0.075): bracketed 0.05..0.1 by cells on both sides (a cut of 0.04 breaks gen-m9's six c = 0.05
  cells). t3349-t-inv-2t-sweep A98 (c 0.0787) and A195 (c 0.0985) come closer from the centre path at a cut of
  0.0999, so the cells favour 0.0985..0.1.
- `T_TAIL_P` (1E-5): it moves 25 cells, all df 1 or 2. Lower cuts are worse; 1E-3 brings m9-t-inv-2t A3
  (`T.INV.2T(0.001,1)`) closer and 1E-2 is worse on net (4 worse, 2 better). The lean to 1E-3 rests on one cell.
- `T_STATRS_MIN_P` (1E-8) moves no cell from 1E-20 to 1E-7 after T-2511 (only df 1 reaches it). It is a guard
  against statrs' hang and panic, so it is not in scope here.
None of these scores passed R4: they are in-sample, on every cell.

## Acceptance criteria
- [ ] For each cut: a hold-out split fixed before scoring (R4.1), train and hold-out counts for the chosen cut and the
      current one, a fresh log-uniform sweep after the freeze (R4.2) and the Gate A envelope result; the chosen cut
      beats the current one on hold-out, or the cut stays and the Outcome says why.
- [ ] The method that moves a cut is committed or its per-cell lists are attached, so the split is auditable.
- [ ] The `FITTED` tags state the new counts and ranges; `make eval-baseline` records the fixes; no regression.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The df bands and extrapolated fits of T-3364.
