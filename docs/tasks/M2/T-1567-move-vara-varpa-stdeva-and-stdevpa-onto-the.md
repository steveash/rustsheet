---
id: T-1567
title: "Move VARA, VARPA, STDEVA and STDEVPA onto the shared aggregate-argument walker"
milestone: M2
lane: quality
status: todo
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
- [ ] `variance_of_values_a` uses the walker, and its own dispatch is gone.
- [ ] `make eval`: 0 regressions and 0 fixes in every suite.
- [ ] scripts/check_duplication.py does not rise.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Behavior changes of any kind.
