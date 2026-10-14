---
id: T-1475
title: "TBILLEQ and TBILLYIELD return Excel's last digits (ironcalc-calc TBILLs.xlsx, 27 cells)"
milestone: M2
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
TBILLEQ and TBILLYIELD give Excel's exact doubles. This clears the TBILL cells of ironcalc-calc and, through
them, the function-free cells that M2's criterion "ironcalc-calc cells using only M1/M2 functions" counts.

## Context
Kernel map (T-1522, decision 0056): rank 15 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
ironcalc-calc/FINANCIAL/TBILLs.xlsx (Excel-cached, Microsoft Macintosh Excel) has 27 failing cells (28 baseline
items). They are 1 to a few ulps off, or NumericClose for D24/D26:
- TBILLEQ: D5, D6, D7, D9, D10, D16, D20, D24, D26, D30;
- TBILLYIELD: R10, R16, R17, R19, R20, R24, R26, R30;
- `=(360-365*C5/D5)/C5` in L5, L6, L7, L9, L10, L16, L20, L24, L26. These cells call no function, so they count
  for M2 under a literal reading of docs/milestones.md (T-1472 measures this). They fail only because their D input does.
Probe evidence (excel-graph) for the same functions: probes/fin-argument-modes H310-H319 (the TBILL base and
argument-mode rows), probes/date-system-1904-financial A29 (TBILLEQ) and A30 (TBILLYIELD), and
probes/date-system-1904-financial-shift A32 (TBILLYIELD).
Both functions are closed-form. The miss is in the evaluation order or the formulation (TBILLEQ has a ≤182-day
branch and a quadratic branch), not in iteration. Fit against the cells above. If they do not pin it, add a
probes-numeric sweep (excel-probe skill), as T-0942 did for powers. Code: `fn_tbilleq` and `fn_tbillyield`
in crates/rustsheet-engine/src/functions/financial/mod.rs.
Split from T-1374, which keeps the other bond and depreciation functions.

## Acceptance criteria
- [x] The TBILLs.xlsx F lines leave eval/baseline/ironcalc-calc.txt, or each remaining one is explained in the semantics entry with evidence.
- [x] The TBILL cells listed above leave eval/baseline/probes.txt, with no new F line in any suite.
- [x] docs/excel-semantics/ (financial entry for TBILLEQ/TBILLYIELD) records the formulation and its evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
- TBILLPRICE unless it fails in the same cases. Other bond functions (T-1374), and the solvers (T-0916).

## Outcome
TBILLYIELD is `(100 - pr)/pr * (360/DSM)`. TBILLEQ's 183-plus-day branch is the closed-form root
written with `price = 100 - rate*DSM*100/360` and `t = DSM/Y`, replacing the general quadratic.
Both were fitted on the Excel-cached cells (docs/excel-semantics/financial.md "TBILLEQ and
TBILLYIELD"). Eval: ironcalc-calc known failures 1474 -> 1447 (TBILLs.xlsx 273/273), probes
-4 (fin-argument-modes H313, date-system-1904-financial A29:A30, -shift A32), with 0 regressions.
New probe tbilleq-zero-price 3/4: its A3 (the ≤182-day zero denominator) is the one new-coverage F
line, filed as T-1540. Follow-up T-1539
(zero-day TBILLYIELD).

