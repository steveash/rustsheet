---
id: T-2064
title: "Per-element callers over an array of references: SUBTOTAL, COUNTIF/SUMIF(S), COUNTBLANK, AREAS, ROW/COLUMN/CELL, ISFORMULA, N/T (SPEC-0014)"
milestone: M6
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1956]
---

## Goal
SUBTOTAL, COUNTIF, SUMIF, SUMIFS, COUNTBLANK, AREAS, ROW, COLUMN, CELL, ISFORMULA, N and T lift over an array of references, one call per whole element, and VSTACK hands the elements on in a reference position, as Excel does.

## Context
Design: docs/specs/SPEC-0014-reference-array-results.md rules 6 and 7, "Consumers" (`Model::lift_over_ref_grid`) and "Bounds" (each element's cells charged before walking). Builds on T-1956's `RefGrid`. Evidence: probes/reference-lift-consumers E57-E78, E97, E109, E113, E165, E169, E185, reference-lift-consumers-b1c E85-E93, -b1d E97, -b2 E125-E133, E177, E193, E197, E225 (excel-graph, 2026-10-19). Today COUNTIF/SUMIF read their range through `evaluate_node_with_reference` (functions/statistical/if_ifs.rs) and SUBTOTAL through `evaluate_picked_reference_arg` (functions/subtotal.rs); both must read `RefArg`.

## Acceptance criteria
- [ ] The cells above pass and leave eval/baseline, with no new known failures.
- [ ] A per-element walk past the lift's visit budget is its bound error before any element runs (unit test).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
