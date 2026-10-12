---
id: T-1304
title: "MATCH: a match_type between -1 and 0 searches descending (floor, not truncation; 2 gen-m2 known failures)"
milestone: M2
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MATCH with a fractional match_type between -1 and 0 (`-0.5`, `-0.0001`) runs the descending
approximate search, as Excel does, instead of an exact match.

## Context
`fn_match` (crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs, about line 308) reads
match_type as `v as i32`, which truncates toward zero, so `-0.5` and `-0.0001` become 0 (exact
match). Excel treats them as negative (descending search). Evidence, excel-graph oracle (gen-m2
sidecars):
- `gen-m2/f7e6d351c8538ef5` Calc!A160 `=(MAX(MATCH(Data!A4,Data!C7:C10,(-0.0001)))>0.3)`.
  A4 is blank (key 0) and C7:C10 = blank, 1E+15, 0.5, "a". Excel gives TRUE: the descending search
  finds 0.5 at position 3. The engine's exact search gives `#N/A` (a blank never matches 0).
- `gen-m2/96a0952412cefa6c` Calc!A241
  `=COUNT(COUNTIF(Data!A10:D10,"<>1"),Data!D4:D10,MATCH(("?b"&Data!C5),Data!B10:B10,IFERROR((-0.5),Data!B3)))`.
  Excel gives 7: COUNTIF is 1, D4:D10 holds 5 numbers, and `MATCH("?bFALSE",{"Hello World"},-0.5)` = 1,
  because "Hello World" >= "?bFALSE" in a descending search. The engine gives 6 (`#N/A`).

`floor` fits every Excel cell: it sends (-1, 0) to -1 and leaves positive fractions where truncation
puts them. No passing cell distinguishes floor from truncation for positive fractions, because
`MATCH(..,0.1)` / `0.3` / `1.005` only appear over 2-D ranges or where both searches agree
(`gen-m2/8eb18149d4245e1a` A300, `04cfd82f54dbc2b2` A199/A387, `2cfe279fe77a000d` A194,
`ef5c55a81783b7d5` A344). A huge match_type (`123456789.123`, `1E+15`) must still be ascending
(`f9d4df4efdfbcbd5` A255, `f0d1bacec5de8146` A16, `207a5753da48c7e0` A314 pass today).
Semantics: docs/excel-semantics/lookup.md "Approximate-match lookups: type-skipping binary search"
already says "Any negative match_type searches descending". The task adds that a fractional one
counts by its floor (or its sign) and cites these cells.

## Acceptance criteria
- [ ] The lines `gen-m2/96a0952412cefa6c.case.json	Calc!A241` and
      `gen-m2/f7e6d351c8538ef5.case.json	Calc!A160` disappear from eval/baseline/gen-m2.txt
      (`make eval-baseline`), with no new F lines in any suite.
- [ ] An engine unit test covers `MATCH(…,-0.5)` (descending), `MATCH(…,0.5)`
      (unchanged: exact, as today) and `MATCH(…,1E+15)` (ascending, no overflow or panic).
- [ ] docs/excel-semantics/lookup.md cites A160 and A241 for the fractional match_type rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Positive fractional match_types (0 < m < 1). Truncation and floor agree there, and no Excel cell
  shows whether Excel uses the sign instead (`MATCH(x,{…},0.5)` ascending?). If the code change
  would alter them, probe first (excel-probe skill); otherwise file a probe follow-up.
- Other MATCH cells in gen-m2 that the rule does not explain: `04cfd82f54dbc2b2` A81 and
  `f9d4df4efdfbcbd5` A14 (a text match_type from a cell, Excel `#REF!`). Also `f7e6d351c8538ef5`
  A334: `MATCH(0.5,{0.1,TRUE,3.14159},-1)` is `#N/A` in Excel, but the engine's descending search
  skips TRUE forward to 3.14159 and returns 3. Skipping backward would fit, but one cell is not
  enough evidence. These cells need their own probe.
- XMATCH and VLOOKUP/HLOOKUP range_lookup.

## Outcome
`fn_match` floors match_type (`v.floor()`), so -0.5 and -0.0001 search descending. Fixes gen-m2 A160 and A241; unit test added; lookup.md cites both cells.
