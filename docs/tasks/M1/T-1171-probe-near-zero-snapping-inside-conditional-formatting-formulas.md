---
id: T-1171
title: "Probe near-zero snapping inside conditional-formatting formulas written in parentheses"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence says whether Excel snaps near-zero results in a conditional-formatting rule formula, and whether a formula written wholly in parentheses is exempt as in a cell.

## Context
T-1164 made `user_formula_to_internal` keep the grouped root of a CF formula the way it does for a defined-name body; numbers.md "Near-zero snapping" marks the CF behavior UNVERIFIED. Probe `=(0.1+0.2-0.3)=0` and `=0.1+0.2-0.3=0` style rules (CF formulas compare, so use a cell-visible effect such as a rule whose fill depends on `=(0.1+0.2-0.3)=0`); if the CF result is not observable through excel-graph, record that and close.

## Acceptance criteria
- [x] A probe case or a recorded reason it cannot be observed; numbers.md updated.
- [x] No engine mismatches observable; desktop follow-up filed as T-1204 (M14).
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
CF rule results are not observable: excel-graph returns cell values only and the case format has no CF rules or rendered-fill output. Recorded in numbers.md (near-zero snapping) and oracle.md (Unobservable features); the desktop-Excel question belongs to M14 Phase 4. No eval delta, no engine change. Follow-up: T-1204 (desktop Excel probe, M14).
