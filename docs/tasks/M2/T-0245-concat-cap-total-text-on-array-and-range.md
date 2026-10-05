---
id: T-0245
title: "CONCAT: cap total text on array and range arguments; array-valued cells inside ranges"
milestone: M2
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
CONCAT bounds the total text it builds from array and range arguments and handles array-valued cells inside ranges, instead of `#N/IMPL!` or unbounded growth.

## Context
Found in T-0222 review: `fn_concat` (functions/text/common.rs) has no total-size cap on its Array/Range branches (the Range branch is also quadratic via `format!`), and a Range containing array-valued cells still returns `#N/IMPL!`. Use `within_text_budget` from functions/array_lift.rs; any cap is a DEVIATION (decision 0014, docs/excel-semantics/text.md).

## Acceptance criteria
- [ ] CONCAT over a huge array/range returns #VALUE! past the budget with a unit test; DEVIATION entry added.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
