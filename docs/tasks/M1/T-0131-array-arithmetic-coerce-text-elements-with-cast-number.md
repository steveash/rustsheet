---
id: T-0131
title: "Array arithmetic: coerce text elements with cast_number (Data!D7:D7/1)"
milestone: M1
lane: engine
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
Arithmetic on a range or array coerces each text element the way a scalar text operand is coerced,
so `Data!D7:D7/1` with D7 = " 2 " gives 2.

## Context
gen-m1 (T-0109 triage, oracle excel-graph 2026-10-05): 10 failing cells. Every one reads a
text-number cell `" 2 "` through a range in arithmetic. The engine gives `#VALUE!`:
- `gen-m1/1ce5b2467b3b19fd` (`range/single-cell-arith`): A14 `Data!D7:D7/1` Excel 2; A99
  `Data!D1:D1*2.5` Excel 5; A142 `Data!C6:C6-2.5` Excel -0.5; A79 `Data!D3:D3/0` Excel `#DIV/0!`;
  A200, A212, A223.
- `gen-m1/9646257ac139cd1e` (`cross-sheet/single-range`): A25 `Data!E9:E9-7` Excel -5; A81
  `'Q''1'!D7:D7*1.5` Excel 3; A175 `Data!D3:D3/10` Excel 0.2.
- Found in T-0105, once the intersection operator parsed: `gen-m1/7cb99acdd82402f1`
  (`whole-row-col/row-cell-arith`) A72 `('Q''1'!5:5 'Q''1'!B5)+2` Excel 4; A103, A217, A235
  (the intersected cell is " 2 ").
The same form over a numeric cell passes (`Data!B2:B2+2`, 1ce5… A135), so the 1x1 range itself
is not the problem. The scalar path already coerces: `-" 2 "*2` = -4 (10f579127ffb6d9b A106) passes.
Semantics: docs/excel-semantics/coercion.md "Text in a range operand of arithmetic is coerced".

Root cause: `arithmetic.rs` `to_f64` (used for every array element in `arithmetic_on_values`)
parses strings with a bare `s.parse::<f64>()`, which does not trim and has none of the
currency/percent/date handling. That duplicates `Model::cast_number` (cast.rs), which the scalar path
uses. Per CLAUDE.md directive 3, route array elements through `cast_number`; do not add a second parser.

## Acceptance criteria
- [ ] The 14 cells above disappear from eval/baseline/gen-m1.txt with no new F lines in any suite.
- [ ] `to_f64` no longer parses text itself (one coercion routine; reviewer-quality checks).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Number→text conversion (T-0107).
