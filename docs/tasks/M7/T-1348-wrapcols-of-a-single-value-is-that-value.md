---
id: T-1348
title: "WRAPCOLS of a single value is that value, not padded to wrap_count; a wrap_count below 1 is #NUM!"
milestone: M7
lane: functions-lookup
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
WRAPCOLS of a single value returns that value (1x1, no `#N/A` padding to wrap_count), and a wrap_count below 1 (e.g. 0.5) is `#NUM!`, as in Excel.

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11); docs/excel-semantics/arrays.md "WRAPCOLS of a single value":
- gen-m7/bb4617644fee40c9 Calc!A30 `=WRAPCOLS((0.1>REPLACE(Data!$C$2,7,Data!D$7,"a*")),4)`: Excel FALSE (one cell, it does not
  spill), engine `#SPILL!` (it pads to 4x1: `ROWS(WRAPCOLS(1,4))` is 4).
- gen-m7/d38b52bfa1a9f9e6 Calc!A26 `=(WRAPCOLS("1.5",4)&DROP(Data!C10,0,3))`: Excel `#CALC!` (1x1 & DROP's `#CALC!`), engine `#SPILL!`.
- gen-m7/f68fafa0ca2eba26 Calc!A39 `=HYPERLINK(WRAPCOLS(Data!C12,0.5,(Data!B2^0.1)),HOUR(DAY(10161)))`: Excel `#NUM!`, engine
  `#VALUE!` (`WRAPCOLS(Data!C12,0.5)` alone is `#VALUE!`).
Code: crates/rustsheet-engine/src/functions/lookup_and_reference/wrapcols_wraprows.rs. Only single values are evidenced.
Before generalising, probe with the excel-probe skill `WRAPCOLS({1,2},4)`, `WRAPCOLS(A1:A2,4)`, `WRAPROWS(1,4)`,
`WRAPROWS({1,2},4)` and wrap_count 0.5, 0, -1 for both functions, and match all of them (the engine pads `WRAPCOLS({1,2},4)` to
4 rows today). Lifting wrap_count/pad_with over an array is T-1835.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/3b8e9078261b5bd2 Calc!A9 `=WRAPCOLS(FIXED(SINH(Data!E12)),4,(-1))`: Excel "1.18" (one cell), engine `#SPILL!`.
- gen-m9/675763979c1defb6 Calc!A6 `=WRAPCOLS(Data!D6:D6,2,VALUE("Hello World"))`: Excel `#VALUE!` (pad_with's error, although a single value needs no padding), engine `#SPILL!`.
- gen-m9/50f10c4062fb2e98 Calc!A21 `=WRAPROWS(Data!C5:C12,(Data!$D12+BESSELI(Data!$D12,Data!C9)))` (wrap_count 0): Excel `#NUM!`, engine `#VALUE!`; WRAPROWS too.

## Acceptance criteria
- [ ] gen-m7/bb4617644fee40c9 A30, gen-m7/d38b52bfa1a9f9e6 A26 and gen-m7/f68fafa0ca2eba26 A39 pass in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] The probe cells pass; arrays.md's entry is VERIFIED.
- [ ] gen-m9/3b8e9078261b5bd2 A9, 675763979c1defb6 A6 and 50f10c4062fb2e98 A21 pass, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
