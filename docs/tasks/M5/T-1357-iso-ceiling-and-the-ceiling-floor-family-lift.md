---
id: T-1357
title: "ISO.CEILING and the CEILING/FLOOR family lift over an array argument (#N/IMPL! today)"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-0222]
---

## Goal
ISO.CEILING, CEILING, CEILING.MATH, CEILING.PRECISE, FLOOR, FLOOR.MATH, FLOOR.PRECISE and MROUND lift element-wise over an array (and a multi-cell range in a dynamic-array formula) in every argument, as Excel does, instead of `#N/IMPL!`.

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11): gen-m7/e5a361112d83d0c3 Calc!A9 `=(Data!B10<>ISO.CEILING((Data!D3/Data!E$8),HSTACK(Data!C1:E1,Data!D11)))`:
Excel TRUE (a 1x4 result that spills right), engine `#N/IMPL!` (`ISO.CEILING(1.5,{1,2})`, `CEILING.MATH(1.5,{1,2})`,
`CEILING(1.5,{1,2})`, `MROUND(1.5,{1,2})` and `FLOOR.MATH(1.5,{1,2})` are all `#N/IMPL!`; `ISO.CEILING(1.5,Data!C1:E1)` is
`#VALUE!`). Only ISO.CEILING is evidenced; the rest follow the general lift rule of docs/excel-semantics/arrays.md. T-0222 lifts
ROUND/ROUNDUP/ROUNDDOWN through the shared helper (functions/text/text_arg.rs lineage): build on it, do not add a second
lifting primitive. Code: crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs. If unsure about a
range argument, add a probe row for `ISO.CEILING(1.5,A1:B1)` with the excel-probe skill.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/89693ab0cafc29df Calc!A14 `=CEILING.MATH((RANK(Data!C4,Data!A5:C12,0)-(Data!C11-7)),Data!C3,(INDEX(Data!D4:D12,Data!D12,1)^AVERAGE(..)))` (mode is a 9x1 array): Excel `#SPILL!` (it lifts even though number is an error: RANK of the TRUE cell C4), engine `#VALUE!`.

## Acceptance criteria
- [ ] gen-m7/e5a361112d83d0c3 A9 passes in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] Unit tests: each listed function with an array in its first and its second argument.
- [ ] gen-m9/89693ab0cafc29df A14 passes, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
