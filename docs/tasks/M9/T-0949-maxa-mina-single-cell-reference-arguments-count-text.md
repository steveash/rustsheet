---
id: T-0949
title: "MAXA/MINA: single-cell reference arguments count text as 0 and booleans as 1/0 (2 poi known failures)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
MAXA and MINA treat a single-cell reference argument like a one-cell range: a text cell counts as 0
and a boolean cell as 1/0. Today `for_each_value_a` skips text and booleans that come from a
single-cell reference, so `MAXA` of only text references is `#VALUE!` and `MINA(TRUE-cell, …)`
ignores the TRUE.

## Context
Evidence: poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15), sheet EverythingTests
(`make eval-show CASE=poi/FormulaEvalTestData_Copy.xlsx`).
- J888 `MAXA(F14,E19,F12)` with F14, E19 and F12 all non-numeric text:
  expected 0; actual `#VALUE!`.
- F912 `MINA(B16,B10,B9)` with B16 = TRUE, B10 = 534, B9 = 2: expected 1; actual 2.
Passing today and must stay so: rows 888 and 912 otherwise, e.g. E912 `MINA(E15:E17)` (text range) = 0,
H912 `MINA("-23",E13,S7)` = -23, F888 `MAXA(B14:B17)` = 1.

Rule recorded in docs/excel-semantics/coercion.md "MAXA / MINA: single-cell references are read like
ranges".

Code: `for_each_value_a` in `crates/rustsheet-engine/src/functions/statistical/count_and_average.rs`;
its `Boolean`/`String` arms are guarded by `!matches!(arg, Node::ReferenceKind { .. })` and the
reference case falls through to `_ => {}`. Only `fn_maxa`/`fn_mina` call `for_each_value_a`, so the
change is local; still, no new F lines anywhere.

## Acceptance criteria
- [ ] These 2 lines disappear from eval/baseline/poi.txt with no new F lines in any baseline:
      `poi/FormulaEvalTestData_Copy.xlsx` EverythingTests!J888, F912.
- [ ] Unit tests: MAXA/MINA of single-cell references to text (0) and to TRUE/FALSE (1/0).
- [ ] coercion.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MAXA/MINA with no arguments counted at all (today `#VALUE!`; no evidence).
