---
id: T-1410
title: "TEXT lifts over an array or multi-cell range format_text"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXT lifts over an array or multi-cell range in its format_text argument (one result per format, a spill), as it
already does over its value argument.

## Context
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/89693ab0cafc29df Calc!A36 `=TEXT(Data!E7,FILTER(Data!B1:E12,(Data!A1:A12<7)))`: Excel `#SPILL!` (an array result
  blocked by the next Calc cells), engine `#N/IMPL!`.
- gen-m9/95efb37a9874f82e Calc!A16 `=TEXT(FLOOR(Data!E11,EOMONTH(37390,12)),(UNIQUE(Data!D9:D12)&(Data!A7&Data!E10)))`: Excel
  `#SPILL!`, engine `#N/IMPL!`.
Engine today: `TEXT({1;2},"0")` and `TEXT(A8:A9,"0")` lift (2 rows); `TEXT(1,{"0";"0.0"})` is `#N/IMPL!` and
`TEXT(1,A8:A9)` is `#VALUE!`. docs/excel-semantics/text.md "Text function argument modes" (scalar arguments lift);
VALUETOTEXT already lifts its format (T-1229): reuse that lifting. Code: `fn_text` (crates/rustsheet-engine/src/functions/text/common.rs).

## Acceptance criteria
- [ ] gen-m9/89693ab0cafc29df A36 and gen-m9/95efb37a9874f82e A16 pass in `./target/release/rustsheet-eval show gen-m9/<case>.case.json`, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] Unit tests: `ROWS(TEXT(1,{"0";"0.0"}))` = 2 and a two-cell format range spills two results.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
