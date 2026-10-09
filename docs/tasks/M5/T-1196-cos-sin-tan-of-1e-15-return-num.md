---
id: T-1196
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

## Acceptance criteria
- [ ] A probe case with oracle values for the threshold and the sibling trig functions.
- [ ] gen-m5 c2ce66769f6cc566 A32 and 065c0542a4ce3d1c A26 pass; baseline updated if the suite gains one.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
