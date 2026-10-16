---
id: T-0429
title: "Score iterative-calculation probes against one calculation: oracle single-calc mode or a harness recalc count"
milestone: M3
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Iterative-calculation probe cells can be scored against an engine that calculates once per recalculation, as desktop Excel does.

## Context
Found in T-0428. The excel-graph oracle calculates each workbook three times (on load, at
`calculate` FullRebuild, and at the scratch-sheet edit; tools/src/rustsheet_tools/oracle/backends.py).
Iteration resumes from the previous values each time, so the cells of probes/circular-iteration-on,
-limits, -delta and -delta-small show three calculations' worth of iterations
(docs/excel-semantics/recalc.md "How excel-graph calculates a probe workbook"). Choose one of:
- an oracle mode that calculates once (e.g. no fullCalcOnLoad, and the edit before the rebuild),
  if Excel still saves the result;
- a case-level setting that tells the eval adapter how many recalculations to run.
Either way the engine must not fake extra recalculations. The single-calculation data would also
pin down the UNVERIFIED delta stopping test in recalc.md.

## Acceptance criteria
- [ ] The iteration probes have expected values for one calculation (re-filled), or the
      harness runs the oracle's number of recalculations for them; the choice is recorded in
      recalc.md.
- [ ] recalc.md's UNVERIFIED delta stopping test is settled or narrowed with the new evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing iterative calculation (T-0401, Phase 2).
