---
id: T-3393
title: "Explain AMORDEGRC left-over state around dynamic-array formulas (probes -ag, -ah)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's behaviour for AMORDEGRC's left-over state around dynamic-array formulas is explained and the engine matches it.

## Context
T-2844 left two known failures: probes/amordegrc-fraction-state-ag (a fraction above `=SEQUENCE(1,2)` above the 485
charge is 0 in Excel, 485 ours; emptying `amor_left_over` at a dynamic-array formula's place in `run_excel_order` did not
change it, so how the pass reaches a spilling anchor needs finding) and -ah (`=SEQUENCE(1,2)+AMORDEGRC(..485..)` in A2
above an AMORLINC in A3 leaves 485 for A1, ours 132; -ai, the same formula with `*0` and no A3, leaves 0). See
financial.md "AMORDEGRC below period 1".

## Acceptance criteria
- [ ] A probe of an error element in a lifted call (`{1,-1,0.5}`) is added. More probes tell where a dynamic-array formula (and the charges inside it) sits in the order; -ag and -ah pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
