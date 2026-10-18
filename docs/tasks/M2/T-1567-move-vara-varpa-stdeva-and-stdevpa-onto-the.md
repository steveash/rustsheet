---
id: T-1567
title: "Move VARA, VARPA, STDEVA and STDEVPA onto the shared aggregate-argument walker"
milestone: M2
lane: quality
status: done
priority: P3
size: S
tier: sonnet
spec: docs/specs/SPEC-0009-aggregate-argument-walker.md
depends_on: [T-1474]
---

## Goal
VARA, VARPA, STDEVA and STDEVPA read their arguments through the SPEC-0009 walker with the A-rules. No result changes.

## Context
SPEC-0009 leaves `Model::variance_of_values_a` (functions/statistical/variance.rs) out of T-1474 and T-1476. This is its own argument dispatch. Map it onto `for_each_aggregate_number` with the A number rules (`CastA` / `CoercedA` / the array rule that matches today's code). Keep today's error order, which is `First` unless the code shows otherwise. Evidence that must keep passing is cited in the poi EverythingTests cells for VARA/STDEVA (coercion.md) and in any probe naming these functions (`rg -l "VARA|STDEVA" eval/suites`).

## Acceptance criteria
- [x] `variance_of_values_a` uses the walker, and its own dispatch is gone.
- [x] `make eval`: 0 regressions and 0 fixes in every suite.
- [x] scripts/check_duplication.py does not rise.
- [x] `make check` green; reviewers approve.

## Out of scope
- Behavior changes of any kind.

## Outcome
`variance_of_values_a` (VARA, VARPA, STDEVA, STDEVPA) now calls `for_each_aggregate_number` with `ErrorRule::First`, `DirectNumbers::NumbersOnly` (removed in T-1291; new here: only a direct number counts; booleans, text and the rest are skipped, as the old code did "for now"), `CellNumbers::CoercedA` and `ArrayNumbers::Numbers`. The now-unused `for_each_coerced_number_in_areas` is deleted. Eval delta 0 fixed, 0 regressions in every suite; duplicated lines unchanged at 6360.

Unprobed differences from the old code, all now recorded in docs/excel-semantics/statistics.md as UNVERIFIED with follow-up T-1617: (1) a single-cell reference (`VARA(A1)`, `(A1)`, `Sheet1!A1`, a LET-bound cell) and a cell picked by IF/INDEX/XLOOKUP/TAKE is read as a reference, so text there counts 0 and TRUE counts 1; before it was skipped, which disagreed with `A1:A1` (the fidelity reviewer measured `VARA(A1,A3,A4)` with A1=TRUE as 8 before and 16 now); (2) a linked-workbook reference follows `evaluate_aggregate_arg`; (3) a referenced cell holding a range is "Unexpected Range" instead of skipped (it cannot occur from a cell value). No eval case covers (1)-(3), hence the 0 eval delta. Direct values and array elements keep the old "numbers only" rule, which Excel does not follow (probes/stat-argument-modes Sheet1!H69, H72, H75, H78 are known failures); T-1617 fixes them.
