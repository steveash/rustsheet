---
id: T-1071
title: "SUBTOTAL accepts a union (SPEC-0004 row 17)"
milestone: M1
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-1062]
---

## Goal
SUBTOTAL(9,..) and SUBTOTAL(109,..) accept a union.

## Context
Oracle (probes/multiarea-subtotal): both give 6 for (A1,B2) with A1=1, B2=5. Engine gives #VALUE!. Spec: docs/specs/SPEC-0004-multi-area-references.md; evidence: docs/excel-semantics/references.md "Multi-area references in functions".

## Acceptance criteria
- [ ] Both cells of multiarea-subtotal pass
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' union support (their own tasks under SPEC-0004).

## Outcome
`subtotal_get_values` walks each area of a union (`reference_ranges`) through the new `subtotal_walk_range`, the range
walk it used before (filtered/hidden rows and nested SUBTOTAL cells are skipped per area). Covers every function number
that reads `subtotal_get_values` (sum, average, max, min, product, variance and deviation families). SUBTOTAL 2/3/102/103
(COUNT/COUNTA) have their own walk and stay `#VALUE!` for a union (unprobed). Unit test `subtotal_of_a_union`. Eval delta:
probes known failures 188 -> 186 (fixed 2: multiarea-subtotal H1, H2), 0 regressions.
