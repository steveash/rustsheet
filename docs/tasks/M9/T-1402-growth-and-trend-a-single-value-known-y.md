---
id: T-1402
title: "GROWTH and TREND: a single-value known_y is one point, an error known_y is that error, and unequal sizes are #REF! before text is #VALUE!"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
GROWTH and TREND treat a single-value known_y as one data point, return an error known_y as that error, and report unequal
known_y/known_x sizes as `#REF!` before rejecting text, as in Excel.

## Context
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/d244411dd90f7fbd Calc!A17 `=IF((ISERR(Data!E$12)>CLEAN("1")),GROWTH(GAMMA(Data!$E7)),..)` (E7 = 100, the condition is TRUE):
  Excel 9.332621544394404e+155, engine `#VALUE!` (`GROWTH(5)` and `GROWTH({5})` are `#VALUE!`). The value also needs
  GAMMA(100) to match: the engine's `GAMMA(100)` is 9.332621544395446e+155, about 1e-13 off 99! (T-0973).
- gen-m9/d244411dd90f7fbd Calc!A21 `=OFFSET(Data!B10:E11,TREND(XNPV((-2.5),1e-15,Data!C12)),0)`: Excel `#NUM!` (XNPV's error
  passes through TREND), engine `#VALUE!` (`TREND(#NUM!)` is `#VALUE!`).
- gen-m9/89693ab0cafc29df Calc!A24 `=(GROWTH(Data!B2:B3,(Data!D1+123456789.123),(Data!D2*Data!B$3),(Data!C$11=Data!A$1))<..)`
  (known_y "?b", 0; known_x one number): Excel `#REF!`, engine `#VALUE!`. The engine already gives `#REF!` for
  `GROWTH({1,2},5)`, so here the text check runs before the size check.
Code: `fn_growth` / `fn_trend` in crates/rustsheet-engine/src/functions/statistical/linest.rs. Whether `GROWTH(5)` is exactly
5 (EXP(LN(5)) may differ in the last digit) is not evidenced: assert only the gen-m9 cells and the error cases.

Semantics: docs/excel-semantics/statistics.md "Array arguments given a scalar, a single cell or an error".

## Acceptance criteria
- [ ] gen-m9/d244411dd90f7fbd A21 and gen-m9/89693ab0cafc29df A24 pass in `./target/release/rustsheet-eval show gen-m9/<case>.case.json`, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `GROWTH(5)` and `TREND(5)` return a number; gen-m9/d244411dd90f7fbd A17 passes once T-0973 lands (note it in the Outcome if it still misses only on GAMMA's digits).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
TREND and GROWTH take one known_y as one point (every prediction is that value), an error known_y or known_x is returned as that error, and the size check (`#REF!`) runs before the text check (`#VALUE!`); LINEST and LOGEST still need two points. gen-m9 98.00% -> 98.20% (d244411dd90f7fbd A21, 89693ab0cafc29df A24 pass); probes known failures 1812 -> 1798 (14 fixed, 0 regressions). gen-m9/d244411dd90f7fbd A17 still waits on GAMMA(100)'s digits (T-0973).
