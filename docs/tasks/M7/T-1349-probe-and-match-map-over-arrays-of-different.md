---
id: T-1349
title: "Probe and match MAP over arrays of different sizes or a single value with an array (#VALUE! today)"
milestone: M7
lane: functions-logic
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MAP over arrays of different sizes, or over a single value and an array, returns the array Excel returns instead of `#VALUE!`.

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11); docs/excel-semantics/arrays.md "MAP over arrays of different sizes":
- gen-m7/a0cbc75358b429b6 Calc!A31 `=MAP(Data!B4:E8,Data!A4:D11,Data!B5:E7,LAMBDA(z,x,acc,Data!A3))` (5x4, 8x4 and 3x4):
  Excel `#SPILL!` (an array result blocked by the next Calc cells), engine `#VALUE!`.
- gen-m7/b5186d194fec0fdc Calc!A13 `=MAP(WEEKDAY(7538),Data!B4:E11,LAMBDA(acc,y,acc))` (a number and 8x4): Excel `#SPILL!`,
  engine `#VALUE!`.
`fn_map` (crates/rustsheet-engine/src/functions/logical/map_reduce.rs) rejects differing dimensions. The element values are
not evidenced: probe them first with the excel-probe skill (`MAP({1;2;3},{10;20},LAMBDA(a,b,a+b))`, `MAP(5,{1,2},..)`,
`MAP({1,2},{1;2},..)`, ranges of different sizes, ROWS/COLUMNS of each, INDEX of the padded corner), expecting the usual
broadcast/`#N/A` padding of arrays.md "Array arithmetic" but not assuming it. Reuse the existing broadcast helper.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/55fc8c4ffbd7c7cc Calc!A33 `=(MAP(Data!B8:E10,Data!A8:C12,Data!A2:C5,LAMBDA(z,v,x,CHIINV(Data!A$1,2)))<..)` (3x4, 5x3, 4x3): Excel `#SPILL!`, engine `#VALUE!`.
- gen-m9/95efb37a9874f82e Calc!A17 `=IF(OR(..),MAP(Data!B6,Data!A7:B12,LAMBDA(w,y,"Hello World"))%)` (a cell and 6x2): Excel `#SPILL!`, engine `#VALUE!`.

## Acceptance criteria
- [ ] gen-m7/a0cbc75358b429b6 A31 and gen-m7/b5186d194fec0fdc A13 pass in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] The probe cells pass; arrays.md's entry is VERIFIED.
- [ ] gen-m9/55fc8c4ffbd7c7cc A33 and 95efb37a9874f82e A17 pass, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
