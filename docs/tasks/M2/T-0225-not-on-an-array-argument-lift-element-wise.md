---
id: T-0225
title: "NOT on an array argument: lift element-wise instead of #VALUE! (2 gen-m2 known failures)"
milestone: M2
lane: functions-logic
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`NOT` given a range or an array (for example a column from `INDEX(range,0,n)`) returns an array of
the same shape, one `NOT` per element, as Excel does. Today it returns `#VALUE!` because it only
reads a scalar boolean.

## Context
Found by the gap triager on 2026-10-05 (gen-m2 cells that T-0208 turned into `#SPILL!` expectations
and that no task owned). Both cells sit in column A of the Calc sheet with the next row occupied, so
Excel's multi-cell result cannot spill and shows `#SPILL!`. A scalar `#VALUE!` would have stayed
`#VALUE!`, so the `#SPILL!` shows that NOT returned an array.
- `gen-m2/75840b9c71a50596` Calc!A29 `=NOT(INDEX(Data!B2:E8,Data!C$10,3))` (Data!C10 is FALSE, so
  INDEX returns the 7-row column D2:D8): expected `#SPILL!`, actual `#VALUE!`.
- `gen-m2/f7e6d351c8538ef5` Calc!A106 `=NOT(INDEX(Data!B3:E9,(Data!B$10-0.1),4))` (Data!B10 is 0.2,
  row 0.1 truncates to 0, so INDEX returns the column E3:E9): expected `#SPILL!`, actual `#VALUE!`.

Checked in a scratch case: with room to spill, the engine's `INDEX(...)` alone spills, while
`=NOT(Data!D2:D8)` and `=NOT(INDEX(...))` give `#VALUE!`.

Semantics: docs/excel-semantics/arrays.md "NOT and CONCATENATE lift over array arguments".
Code: `fn_not` in `crates/rustsheet-engine/src/functions/logical/and_or_xor_not.rs` uses
`get_boolean`. Reuse the existing array helpers rather than writing a new one: `get_value_or_array`
in `cast.rs` (and see how `apply_number_unary` lifts numeric unary functions). Each element is
cast with the same scalar rule NOT uses today (`cast_to_bool`); an error element stays that error.

## Acceptance criteria
- [x] `F	gen-m2/75840b9c71a50596.case.json	Calc!A29` and
      `F	gen-m2/f7e6d351c8538ef5.case.json	Calc!A106` disappear from eval/baseline/gen-m2.txt
      (`make eval-baseline`), with no new F lines in any baseline.
- [x] Unit tests: `NOT({TRUE,FALSE})` is `{FALSE,TRUE}`; `NOT(A1:A2)` over TRUE and 0 spills
      FALSE and TRUE; a scalar `NOT(TRUE)` is unchanged.
- [x] docs/excel-semantics/arrays.md entry gets an `Implemented in:` line.
- [x] `make check` green; reviewers approve.

## Out of scope
- AND/OR/XOR (they aggregate, they do not lift; T-0215 covers their error handling).
- Unary minus and `%` on arrays (`=-Data!D3:E3` is `#VALUE!` today): the upstream `--A1:B4`
  port in T-0920.
- Comparisons over arrays that hold errors (T-0129).

## Outcome
`fn_not` evaluates its argument once: a scalar is cast with `cast_to_bool`; a range (bounded by the shared
`check_array_size` allocation limit, as the IS* lift does) or an array is cast element by element (errors stay errors).
NOT's static-analysis result now follows its arguments (`scalar_arguments`) like ABS/TRIM, so an array result is not
collapsed by implicit intersection. gen-m2 known failures 477 -> 475 (A29 and A106 fixed, 0 regressions).
The per-element cast of text, empty and error elements is ASSUMED (not oracle-backed); follow-up probe T-0260.
