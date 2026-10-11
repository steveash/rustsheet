---
id: T-1256
title: "Lookup functions read a reference-valued lookup_value (CHOOSE, OFFSET, INDIRECT, a 1-cell range) as its cell value"
milestone: M2
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
VLOOKUP, HLOOKUP, MATCH, XMATCH, XLOOKUP and LOOKUP use the value of a reference-valued lookup_value (`CHOOSE(1,A8)`, `OFFSET(A8,0,0)`, `INDIRECT("A8")`, `A8:A8`) instead of returning `#N/A`.

## Context
T-1239's gen-m6 suite (excel-graph sidecars, 2026-10-09): gen-m6/963a217976538d5d Calc!A32
`=(ARRAYTOTEXT(Data!A3:C11)&HLOOKUP(CHOOSE(1,Data!D7),Data!A8:A10,AVERAGE(Data!$D9,Data!C4:C5),(7<>1.005)))`: Excel `#VALUE!`
(123456789.123 is found by the approximate search in A8 = 42, then row index -0.0001 is `#VALUE!`), engine `#N/A`.
Reduced with the engine: `HLOOKUP(Data!D7,Data!A8:A10,1,TRUE)` = 42 but `HLOOKUP(CHOOSE(1,Data!D7),..)` is `#N/A`; so are
`VLOOKUP(CHOOSE(1,A8),A8:B10,2,FALSE)`, `VLOOKUP(OFFSET(A8,0,0),..)`, `VLOOKUP(INDIRECT("Data!A8"),..)`, `VLOOKUP(A8:A8,..)`,
`MATCH(CHOOSE(1,A8),A8:A10,0)`, `XMATCH(..)`, `XLOOKUP(..)` and `LOOKUP(..)`. `IF(TRUE,A8)` and `INDEX(A8:A9,1)` work.
Cause: `vlookup_impl` (crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs) takes
`evaluate_node_in_context(&args[0])`, which returns a `CalcResult::Range` for a reference-returning function, and the
search compares that range with each cell. Find the shared lookup_value path and dereference a single-cell range there
once (directive 3), for all six functions.
Only the CHOOSE shape has Excel evidence; add an excel-probe case for the OFFSET, INDIRECT and `A8:A8` shapes (found
and not found, exact and approximate) before relying on them. A multi-cell range lookup_value is T-0270's (lifting).

## Acceptance criteria
- [ ] gen-m6/963a217976538d5d A32 passes in `./target/release/rustsheet-eval show <case>`, with no new failing gen-m6 cell
      and no new known failures in eval/baseline/.
- [ ] An excel-graph probe case covers the six functions with CHOOSE, OFFSET, INDIRECT and 1-cell-range lookup values and passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Multi-cell range or array lookup values (T-0270).
