---
id: T-1190
title: "MULTINOMIAL reads numeric text and matches Excel's inexact result (9.999999999999998 for 2,3)"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
MULTINOMIAL returns Excel's exact doubles, including its inexact ones, and reads numeric text the way Excel does.

## Context
T-1082's probe probes/math-argument-modes (excel-graph, 2026-10-08) shows Excel's MULTINOMIAL is not
an exact factorial ratio: `MULTINOMIAL(2,3)` = 9.999999999999998 (exact 10), `MULTINOMIAL(4,6)` =
209.99999999999977 (exact 210), `MULTINOMIAL(4,2,6)` = 13859.99999999998; `MULTINOMIAL(2.5,3)`
truncates like `(2,3)`. Numeric text is a number both typed (`MULTINOMIAL("2",3)`, `("4",6)`) and in a
range cell (I1:I3 holding 4, "2", 6), while text and booleans in a range or typed are `#VALUE!`
and a blank cell reads as 0. The engine returns the exact value and `#VALUE!` for numeric text
(failing cells C9, C35, C47, C49:C53). Finding Excel's algorithm is numeric fitting: probe more
(many small and large argument sets, one and three arguments) and test candidates (a running
product of COMBIN-like ratios, exp(gammaln), division order) against every cell.
Code: `fn_multinomial` in crates/rustsheet-engine/src/functions/math_and_trigonometry/multinomial.rs.
Semantics: docs/excel-semantics/numbers.md "MULTINOMIAL is inexact"; coercion.md "GCD / LCM / MULTINOMIAL range cells".

More inexact values (T-1197, probes/multinomial-large-arguments, excel-graph 2026-10-09): `MULTINOMIAL(170)` is
1.0000000000001974 (exact 1, so it is not the plain ratio even for one argument). Also A13 `(169,1)` 170.0000000000337,
A14 ten 1s 3628800.0000000084, and A7, A11, A20; these are its six known failures.

## Acceptance criteria
- [ ] probes/math-argument-modes C9, C35, C47, C49:C53 pass, plus a new probe of 20+ argument sets.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GCD/LCM (they already match).
