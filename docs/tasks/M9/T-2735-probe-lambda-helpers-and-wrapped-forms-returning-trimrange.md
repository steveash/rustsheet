---
id: T-2735
title: "Probe LAMBDA helpers and wrapped forms returning TRIMRANGE of a range"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence says whether a LAMBDA step that returns TRIMRANGE of a range is a reference outside the direct REDUCE body that T-2596 covers.

## Context
T-2596 made REDUCE keep a body that is `TRIMRANGE(range)` as the trimmed reference
(probes/t2579-array-value-edges B49; `LambdaCaller::keeping_trimrange_references`, functions/logical/lambda.rs).
Unprobed (docs/excel-semantics/arrays.md "Array-value edges in the gen-m9 sweep"): a TRIMRANGE wrapped in IF, LET or
CHOOSE as REDUCE's body (a grouped `(TRIMRANGE(r))` is kept as the reference since T-2596, also unprobed); SCAN, MAP, BYROW and BYCOL bodies returning TRIMRANGE; a direct call
`LAMBDA(x,TRIMRANGE(x))(D!B1:B3)`. Probe with consumers that tell a reference from an array: `ROWS(IMSQRT(..))`, `ISREF(..)`.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes with oracle values covers each form above.
- [ ] arrays.md records the result; implementation tasks filed via `make new-task` for each form we get wrong.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the forms (follow-up tasks).
