---
id: T-1452
title: "Probe nested reference pass-through for MATCH match_type and share the reference-context dispatch"
milestone: M2
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Nested and range forms of MATCH's match_type (IFS(TRUE,IF(TRUE,D1)), IF(TRUE,D1:D2), 3-D spans, SWITCH default branch) are probed against Excel, and the reference-context dispatch in `match_type_arg` shares one definition with `evaluate_reference_arg` where the oracle allows.

## Context
T-1399 added IF/IFS/SWITCH/LET pass-through in `match_type_arg` (lookup_and_reference/mod.rs) beside `evaluate_reference_arg` (functions/util.rs) and `eval_ref_or_index`. Reviewers flagged the duplicated dispatch and the unprobed forms. `SUM(IFS(TRUE,A1:B2))` must not change (docs/excel-semantics/logic.md).

## Acceptance criteria
- [ ] Probe cells for the forms above, oracle-filled, recorded in docs/excel-semantics/lookup.md Rule 3.
- [ ] One shared reference-context dispatch, no change to aggregate behavior.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed `probes/match-type-nested-passthrough` (17 cells, excel-graph): nesting, the SWITCH default and 3-D spans pass the cell through, and a multi-cell range is its top-left cell, plain or picked. Fixed 9 mismatches. `match_type_arg` now uses the shared `evaluate_chosen_reference` (also behind IFS/SWITCH `as_reference`); aggregates keep `evaluate_reference_arg` unchanged, since IFS/SWITCH there stay values. Recorded in lookup.md Rule 3.
