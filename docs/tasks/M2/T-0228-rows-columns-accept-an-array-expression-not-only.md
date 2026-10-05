---
id: T-0228
title: "ROWS/COLUMNS accept an array expression, not only a reference"
milestone: M2
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`ROWS(x)` and `COLUMNS(x)` return the row/column count of any array value `x` (an array constant or
the array result of an expression), not only of a reference. Today `fn_rows`/`fn_columns` call
`get_reference` and return `#VALUE!` for a non-reference argument.

## Context
Evidence (oracle excel-graph, 2026-10-05, found by T-0209): probes/error-operand-broadcast
C1 `=ROWS(NA()-{1;2;3})` = 3, C3 `=ROWS({1;2;3}/NA())` = 3, C6 `=ROWS(NA()+A1:A3)` = 3,
C7 `=ROWS("abc"*{1;2})` = 2, C8 `=COLUMNS(1/0+{1,2,3,4})` = 4, C10 `=ROWS(A1:A3^NA())` = 3; all six are
in eval/baseline/probes.txt today (actual `#VALUE!`). Note the arrays here are all-error: ROWS counts
the array, it does not propagate element errors. Semantics: docs/excel-semantics/arrays.md
("A scalar error operand broadcasts against an array operand"). Code:
`crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs` (`fn_rows`, `fn_columns`).
Unprobed: a scalar non-reference argument (`ROWS(5)`, `ROWS(NA())`); probe before deciding it.

## Acceptance criteria
- [ ] The six probes/error-operand-broadcast cells above leave eval/baseline/probes.txt, no new F lines.
- [ ] Unit tests for ROWS/COLUMNS over an array constant and an array-valued expression.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ROW/COLUMN (they need a reference and behave differently).

## Outcome
ROWS/COLUMNS now count reference or array values (`rows_columns_extent`); scalar non-reference arguments keep the old error (unprobed). probes known failures 27 -> 21 (6 fixed, 0 regressions). Rule recorded in docs/excel-semantics/arrays.md.
