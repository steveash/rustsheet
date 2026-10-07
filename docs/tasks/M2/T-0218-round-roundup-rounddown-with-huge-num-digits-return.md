---
id: T-0218
title: "ROUND/ROUNDUP/ROUNDDOWN with huge num_digits return a number, not #NUM! (12 gen-m2 known failures)"
milestone: M2
lane: functions-math
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
ROUND, ROUNDUP and ROUNDDOWN with a num_digits of very large magnitude return what Excel returns (the
number, or 0), never `#NUM!`.

## Context
gen-m2 (T-0203): 12 failing cells, actual `#NUM!` in every one. Evidence (excel-graph):
- `gen-m2/f7e6d351c8538ef5` Calc!A35 `=ROUNDDOWN(Data!E3,123456789.123)` (E3 = 0.5): expected 0.5.
- `gen-m2/2cfe279fe77a000d` Calc!A197 `...ROUNDDOWN(42,123456789.123)`: expected 42.
- `gen-m2/893302d6c5b99e2e` Calc!A136 `=ROUND(Data!$B3,ROUND(123456789.123,6))` (B3 = 1e-15): expected 1e-15.
- `gen-m2/1b6ceed7489e0800` Calc!A337 `=ROUNDDOWN(Data!E1,-123456789.323)` (E1 = TRUE): expected 0.
- `gen-m2/7469c3e920e1ade4` Calc!A336 `=MAX(ROUNDDOWN(0.5,SUM(...)))` with digits 1000000000000005.8: expected 0.5 (actual 0).
- **Inconsistent and must be probed**: `gen-m2/caa981b0753f455c` Calc!A270
  `=ROUND((SUM(...)/ROUND(1000000000000000.0,Data!$D$7)),-2)` with D7 = 1E+15 is `#DIV/0!`, so
  `ROUND(1E+15, 1E+15)` is 0. `gen-m2/dd99be1f2454159f` Calc!A159 (`ROUND(-1000000000000001, 1E+15)`)
  is also 0. Yet digits 1000000000000005.8 keep 0.5 (above). Use the excel-probe skill to find the rule
  (an integer-conversion wrap of num_digits is one hypothesis) before implementing it. Do not guess.
- Other cells: `gen-m2/61cacc22c4244179` Calc!A79 A167 A285, `gen-m2/b4392a6514a36ade` Calc!A135,
  `gen-m2/04cfd82f54dbc2b2` Calc!A46 (also lookup), `gen-m2/fa437879253827be` Calc!A372 (also T-0216).

Code: ROUND family in `crates/rustsheet-engine/src/functions/math_and_trigonometry/`. Relates to T-0106
(decimal-correct halves). Coordinate, keep one rounding helper.

## Acceptance criteria
- [ ] A probe case pins the num_digits rule. docs/excel-semantics/numbers.md has the rule (VERIFIED).
- [ ] The 12 cells above disappear from eval/baseline/gen-m2.txt (the two shared ones once their other task lands), with
      no new F lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit differences of ROUND results (T-0106).
