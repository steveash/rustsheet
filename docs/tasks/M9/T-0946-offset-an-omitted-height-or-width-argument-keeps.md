---
id: T-0946
title: "OFFSET: an omitted height or width argument keeps the reference's size (4 poi known failures)"
milestone: M9
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
In `OFFSET(reference, rows, cols, [height], [width])`, a height or width argument that is present
but empty (`OFFSET(K7:L8,0,0,1,)`, `OFFSET(K7:L8,0,0,,1)`, `OFFSET(K7:L8,0,0,)`) means "the
reference's height/width", the same as leaving it out. Today the empty argument counts as 0 and
OFFSET returns `#REF!`.

## Context
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15), sheet EverythingTests,
row 1008 (`make eval-show CASE=poi/FormulaEvalTestData_Copy.xlsx`). K7:L8 = {1, 1.001; 2, 2.9999}.
- I1008 `SUM(OFFSET(K7:L8, 0,0,1,))`: 2.001 (one row, two columns); actual `#REF!`.
- J1008 `SUM(OFFSET(K7:L8, 0,0,,1))`: 3 (two rows, one column); actual `#REF!`.
- M1008 `SUM(OFFSET(K7:L8, 0,0,,))`: 7.0009 (whole K7:L8); actual `#REF!`.
- N1008 `SUM(OFFSET(K7:L8, 0,0,))`: 7.0009 (4 arguments, empty height); actual `#REF!`.
Passing today and must stay so: H1008 `(…,1,1)` = 1, K1008 `(…,1)` = 2.001, L1008 `(K7:L8,0,0)` = 7.0009.

Rule recorded in docs/excel-semantics/references.md "OFFSET: an empty height or width argument
means the reference's size".

Code: `fn_offset` in `crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs`. It reads
height/width with `get_number`, which turns `Node::EmptyArg` into 0, so `height - 1 == -1` → `#REF!`.
Treat an `EmptyArg` height/width like the missing argument (the `l == 3` branch values).

## Acceptance criteria
- [x] These 4 lines disappear from eval/baseline/poi.txt with no new F lines in any baseline:
      `poi/FormulaEvalTestData_Copy.xlsx` EverythingTests!I1008, J1008, M1008, N1008.
- [x] Unit tests in `crates/rustsheet-engine/src/test/test_fn_offset.rs` for the four forms above.
- [x] references.md entry gets an `Implemented in:` line.
- [x] `make check` green; reviewers approve.

## Out of scope
- Empty `rows`/`cols` arguments (no evidence; they already read as 0).
- Negative height/width (the code comment notes a known difference; separate task if evidence appears).

## Outcome
New `offset_size` helper: a missing or empty (`EmptyArgKind`) height/width keeps the reference's size. poi known failures 932 -> 928 (I1008, J1008, M1008, N1008), no regressions; unit test `fn_offset_empty_height_width`.
