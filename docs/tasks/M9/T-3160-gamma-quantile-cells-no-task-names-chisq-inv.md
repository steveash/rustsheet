---
id: T-3160
title: "Gamma quantile cells no task names: CHISQ.INV at df 1E10 and p 1E-200, CHIINV(0.975,5), GAMMA.INV at shape 1E-300 (7 visibly-off gen-m9 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
campaign: semantics
depends_on: []
---

## Goal
The seven visibly-off gen-m9 gamma-quantile cells below show Excel's value (or error), each traced to the kernel
step that gives it.

## Context
- Gap triage 2026-10-29 (opus refill; `scripts/exit_owners.py --all-off` on the 2026-10-29 report, 5d089378). No
  open task names these cells. Several open tasks own nearby regions, so the first step is diagnosis: which of
  them is the root cause, and either fix it here or route the cell by name into that task (an owner must name the
  case beside the cell, T-0099):
  T-2561 (deep-tail CHISQ.INV/GAMMA.INV kernel), T-3070 (shape 1E9 to 2E10 and above the chi-square df cap, the
  #NUM! band), T-3013 (GAMMA.INV at a shape below 1 and a tiny p), T-3131 / T-3016 (T-0972's remaining cells),
  T-2695 (CHIINV last digits).
- Cells (excel-graph; Excel vs ours):
  - gen-m9/8f2ac25945af87da Calc!A240 `CHISQ.INV(1E-16,10000000000.5)`: 9998837266.383266 vs `#NUM!` (wrong type:
    the semantics stage; Excel answers at df 1E10+0.5, so our df or tail cap refuses too early. Read T-3070's
    #NUM! band first).
  - gen-m9/b4173fe25109a238 Calc!A252 `CHISQ.INV(1E-200,9)`: 1.732023266119695E-44 vs 1.7320232661196884E-44.
  - gen-m9/c2974c061384fdd9 Calc!A355 `CHISQ.INV(1E-20,10)`: 0.0005210568413974334 vs 0.0005210568413974328.
  - gen-m9/4ab2ba98d26c8a5e Calc!A85 `CHIINV(0.975,5)`: 0.8312116134866638 vs 0.831211613486663.
  - gen-m9/146b46e74f2dac3b Calc!A152 `GAMMA.INV(0.5,1E-300,ROUNDUP(2^53+1,4))`: 0 vs 2.2250738585072014E-308 (Excel
    flushes to 0 where we return the smallest normal: check statistics.md's subnormal entries and decision 0042).
  - gen-m9/60b0b6e69ad35a45 Calc!A210 `GAMMAINV(0.5,MINVERSE(INT(60)),1.5E+10)` (shape 1/60):
    7.404834485321526E-09 vs 7.404834485321458E-09.
  - gen-m9/c2974c061384fdd9 Calc!A179 `GAMMAINV(0.001,0.01,SUMPRODUCT(Data!C2:D9))`: 6.988576892175477E-293 vs
    6.988576892175518E-293.
  The last three are reported "unattributed" because the inner MINVERSE / ROUNDUP / SUMPRODUCT are not glue; the
  inner values are exact, so the quantile is the cause.
- Code: crates/rustsheet-engine/src/functions/statistical/ gamma quantile (`rg -n 'fn gamma_inv'`).
- Opus: the evidence is thin (one cell per region) and the cells straddle four owners' models; a fit here must not
  undo theirs.

## Acceptance criteria
- [ ] Each of the 7 cells is either fixed here (its known-failure line leaves eval/baseline/gen-m9.txt) or named,
      with its case and cell, in the open task whose kernel causes it, with the reason in the Outcome.
- [ ] No new known-failure lines in any baseline (`make eval-baseline`).
- [ ] Any quirk found (the 1E10+0.5 df answer, the flush to 0 at shape 1E-300) is recorded in
      docs/excel-semantics/statistics.md citing the cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The p = 0.999 band (T-3146, T-3148) and the rest of the regions the tasks above already own.

## Visible-parity stage
Decision 0083 and decision 0114: done when every cell this task keeps is within half a unit of the 15th significant
digit of Excel's value (and the A240 cell is a number, not `#NUM!`), and every cell it routes is named in its new
owner. Stage `semantics` for the wrong-type A240 and the flush-to-0 A152; the numeric cells ride along (if the
diagnosis shows the engine is far from a correctly rounded value where Excel is close, note `accuracy` for them in
the routed task).
