---
id: T-1831
title: "Move the shared two_numbers kernel reader out of financial/ beside the lift helpers"
milestone: M9
lane: quality
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`Model::two_numbers` (two required arguments read with `cast_number_no_bools_ref`, the first one's
error winning) lives beside the shared lift helpers, not in a function category.

## Context
T-1802 made it `pub(crate)` in functions/financial/rate_conversions.rs so QUOTIENT
(math_and_trigonometry/mathematical.rs) could reuse it instead of copying it; its architecture
review asked to move it to a neutral home before a third caller appears. T-1290 moves
`number_or_error` into functions/lift, so put `two_numbers` with it. Pure move: no behavior change.

## Acceptance criteria
- [ ] `two_numbers` is defined in functions/lift (or cast.rs) and every caller uses it from there.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other kernel readers.

## Outcome
`Model::two_numbers` moved unchanged from functions/financial/rate_conversions.rs to
functions/lift/mod.rs; its five callers (EFFECT, NOMINAL, DOLLARDE, DOLLARFR, QUOTIENT) resolve it
through `Model`, so none changed. No behavior change, no eval delta.
