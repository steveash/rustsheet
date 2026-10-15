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
`variance_of_values_a` (VARA, VARPA, STDEVA, STDEVPA) now calls `for_each_aggregate_number` with `ErrorRule::First`, `DirectNumbers::NumbersOnly` (new: only a direct number counts; booleans, text and the rest are skipped, as the old code did "for now"), `CellNumbers::CoercedA` and `ArrayNumbers::Numbers`. The now-unused `for_each_coerced_number_in_areas` is deleted. Eval delta 0 fixed, 0 regressions in every suite; duplicated lines unchanged at 6360.

Two forms are unprobed and now follow the walker's reading instead of `evaluate_node_in_context`: a cell picked by IF/INDEX/XLOOKUP is read as a reference (text 0, booleans 1/0), and a linked-workbook reference follows `evaluate_aggregate_arg`. No eval case covers either; a probe of VARA over those forms is the follow-up (the direct-boolean and direct-text rules are also still unprobed: "A semantics to be added").
