---
id: T-3315
title: "FORMULATEXT of a formula inside its own table shows [@Col], not T[[#This Row],[Col]]"
milestone: M2
lane: engine
status: dropped
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
FORMULATEXT of a table-body cell whose formula reads its own row shows the in-table form `=[@Val]*2`, as Excel does.

## Context
Found by T-2969: probes/t2969-calculated-column-load.case.json I2 expects `=[@Val]*2` for `=T[[#This Row],[Val]]*2` stored in D2 of table T (excel-graph); we return `=T[[#This Row],[Val]]*2`. Related to (not a duplicate of, as it is the engine-side `[@Col]` rendering for a formula in its own table) T-1683; check it / T-2763 for related FORMULATEXT structured-ref rules before changing.

## Acceptance criteria
- [ ] I2 of the case above passes and leaves the probes baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome (dropped, 2026-10-30, T-2969)
Superseded by T-3241 (merged to main): probes/t2969-calculated-column-load I2 passes there, so the baseline entry is gone and nothing is left to implement.
