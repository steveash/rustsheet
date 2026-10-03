---
id: T-0919
title: "ROW/COLUMN of whole-column references: avoid materialising 1M-row arrays"
milestone: M9
lane: functions
status: todo
priority: P2
size: M
depends_on: [T-0911]
---

## Goal
`ROW(A:A)` builds a `Vec<Vec<ArrayNode>>` of 1,048,576 one-element rows, and `@ROW(A:A)` builds it
only to keep the top-left value. Add a first-element fast path for implicit intersection over
ROW/COLUMN and a compact representation (or cap) for large spans, with before/after timing and
peak memory for `ROW(A:A)`.

## Context
Raised by the performance review of T-0911. Size is bounded by the fixed sheet dimensions (so not an
unbounded allocation) and Excel evaluates the same vector, but cost recurs per recalculation. The
fix touches `model.rs`/`array` representation, both size-gated, so it needs its own change.
Also from T-0911 review: `fn_row`/`fn_column` share min/max/single-or-vector logic that could be one
helper, and the other private `ArrayNode`->`CalcResult` copies were already consolidated.

## Acceptance criteria
- [ ] `@ROW(A:A)` evaluates without allocating the full array.
- [ ] Measurements recorded for `ROW(A:A)` before and after.
- [ ] `make check` green.
