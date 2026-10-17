---
id: T-1350
title: "BYROW and BYCOL of a single value treat it as a 1x1 array (#VALUE! today)"
milestone: M7
lane: functions-logic
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
BYROW and BYCOL take a single value (a number, text or the result of a scalar function) as a 1x1 array and call the LAMBDA once, as Excel does, instead of `#VALUE!`.

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11); docs/excel-semantics/arrays.md "BYROW and BYCOL of a single value":
- gen-m7/93976dd9a3578ed9 Calc!A31 `=("ü"&BYROW(COUNT(Data!C3,Data!E4:E10,Data!A1:A6),LAMBDA(z,z)))`: Excel "ü7", engine `#VALUE!`
  (`BYROW(7,LAMBDA(z,z))`, `BYROW({7},..)` and `BYCOL(7,..)` are all `#VALUE!`; MAP, SCAN and REDUCE of 7 already work).
- gen-m7/59cc50701f4937c9 Calc!A4 `=BYROW("",_xleta.PRODUCT)`: Excel 0 (PRODUCT of the row {""}). This one also needs T-1273
  (`_xleta.` names); with LAMBDA it is `BYROW("",LAMBDA(r,PRODUCT(r)))`, which the unit tests can use.
Code: `fn_byrow`/`fn_bycol` in crates/rustsheet-engine/src/functions/logical/bycol_byrow.rs.

## Acceptance criteria
- [ ] gen-m7/93976dd9a3578ed9 A31 passes in `./target/release/rustsheet-eval show gen-m7/<case>.case.json` (and 59cc50701f4937c9 A4 once T-1273 has landed), with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] Unit tests for BYROW and BYCOL of a number, a text and a 1x1 array constant.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
