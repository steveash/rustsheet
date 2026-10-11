---
id: T-1201
title: "COS/SIN/TAN of 1E+15 return #NUM! in Excel (gen-m5 mismatch)"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
COS (and, once probed, SIN/TAN) of a huge argument such as 1E+15 return Excel's `#NUM!`.

## Context
From T-1192's gen-m5 run (excel-graph oracle): `COS(1E+15)` is `#NUM!` in Excel; the engine computes a number. Failing cells: gen-m5/c2ce66769f6cc566 Calc!A32 and gen-m5/065c0542a4ce3d1c Calc!A26 (the error propagates before other errors). First probe the threshold and SIN/TAN/COT/SEC/CSC behavior with the `excel-probe` skill and record it in docs/excel-semantics/numbers.md.

More cells (T-1239's gen-m6 suite, excel-graph, 2026-10-09): gen-m6/3ee7ffc8d6ab061f Calc!A2
`=((SUM(Data!B4:B9)*(-1))-SEC(1000000000000000.0%))` is `#NUM!` in Excel, so `SEC(1E+13)` is `#NUM!` (engine -43.54);
gen-m6/9bd0bd18636e5f9b Calc!A4 `=SECH(CHOOSE(SIN(1000000000000000.0),Data!A$10))` is `#NUM!`, so `SIN(1E+15)` is `#NUM!`
(engine `#VALUE!`: SIN returns 0.858 and CHOOSE rejects the index).

## Acceptance criteria
- [ ] A probe case with oracle values for the threshold and the sibling trig functions.
- [ ] gen-m5 c2ce66769f6cc566 A32 and 065c0542a4ce3d1c A26 pass; baseline updated if the suite gains one.
- [ ] In `(A-COS(1E+15))=INDEX(B11:D11,6,2)` the left operand's `#NUM!` wins over INDEX's `#REF!` (error precedence).
- [ ] gen-m6/3ee7ffc8d6ab061f A2 and gen-m6/9bd0bd18636e5f9b A4 pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
