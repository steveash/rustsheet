---
id: T-1269
title: "VLOOKUP/HLOOKUP with an array col_index or range_lookup use the first element (#N/IMPL! today)"
milestone: M2
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
VLOOKUP/HLOOKUP with an array col_index/row_index or range_lookup give Excel's result instead of `#N/IMPL!`.

## Context
A first oracle fill of probes/vlookup-array-lookup-value (T-0270) showed `VLOOKUP({2;3},A1:B4,{1,2},FALSE)`
spilling only 2; 3 (the index array looks unlifted, its first element used) and `{FALSE;TRUE}` as
range_lookup giving "two"; "three"; the engine gives `#N/IMPL!` from `lookup_index_arg`/`get_boolean`. Probe a scalar lookup
value with an array index, a range index and a non-first element that differs before changing
code (lookup.md "VLOOKUP / HLOOKUP lift over an array lookup_value", T-0270).

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/4629e931955aa4b8 Calc!A29 `=HLOOKUP(NOT(BYCOL(Data!D7,LAMBDA(x,x))),Data!B10:C10,TIMEVALUE(CHOOSEROWS(Data!A2:A7,-1,2)))` (row_index a 2x1 array of `#VALUE!`; the lookup value FALSE is not in B10:C10): Excel `#N/A`, engine `#N/IMPL!`.

## Acceptance criteria
- [ ] Probe case filled by excel-graph; rule recorded in lookup.md. Also probe an error/text index or range_lookup with an array lookup_value (one error or an array of errors?) and blank lookup elements.
- [ ] Both formulas (moved out of T-0270's case) pass in a new probe case; no new known failures.
- [ ] gen-m7/4629e931955aa4b8 A29 passes. (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] `make check` green; reviewers approve.

## Out of scope
- XLOOKUP/MATCH.
