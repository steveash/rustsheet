---
id: T-1359
title: "Probe and match SERIESSUM with x = 0: #NUM! only for a zero or negative power"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SERIESSUM with x = 0 gives Excel's result: `#NUM!` only when a term has a zero (or negative) power, otherwise the sum (0 for positive powers).

## Context
Excel evidence: crates/rustsheet-xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/SERIESSUM.xlsx C1 `SERIESSUM(0,0,2,{1,2,3})` and
H7 `SERIESSUM(0,0,1,D9:D14)` are `#NUM!` (first power 0). T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11): gen-m7/d38b52bfa1a9f9e6 Calc!A27
`=IFS((SERIESSUM(Data!C$1,Data!A$2,Data!$B10,Data!A$2)>UPPER("TRUE")),..,CHOOSEROWS(UPPER("TRUE"),(0.1-Data!A7),-1,0),..)` with
C1 blank, A2 = 123456789.123, B10 = -0.5: Excel `#VALUE!`, engine `#NUM!`. The engine's SERIESSUM returns `#NUM!` for any x = 0
(seriessum.rs: `if x == 0.0`), so IFS stops at the first condition; in Excel SERIESSUM(0, 123456789.123, -0.5, 123456789.123)
is not `#NUM!` (one positive power), the first condition is FALSE and the second, CHOOSEROWS, gives `#VALUE!`.
docs/excel-semantics/numbers.md "SERIESSUM with x = 0" (UNVERIFIED). Probe with the excel-probe skill: `SERIESSUM(0,1,1,{1,2})`,
`(0,2,-1,{1,2,3})` (powers 2, 1, 0), `(0,1,-2,{1,2})` (powers 1, -1), `(0,0.5,1,1)`, `(0,-1,1,1)`, a blank x, then match.
Code: `fn_seriessum` in crates/rustsheet-engine/src/functions/math_and_trigonometry/seriessum.rs.

## Acceptance criteria
- [ ] gen-m7/d38b52bfa1a9f9e6 A27 passes in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/ (ironcalc-calc SERIESSUM.xlsx C1 and H7 keep passing).
- [ ] The probe cells pass; numbers.md's entry is VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
