---
id: T-1190
title: "MULTINOMIAL reads numeric text and matches Excel's inexact result (9.999999999999998 for 2,3)"
milestone: M5
lane: functions-math
status: done
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
- [x] probes/math-argument-modes C9, C35, C47, C49:C53 pass, plus a new probe of 20+ argument sets.
- [x] `make check` green; reviewers approve.

## Out of scope
- GCD/LCM (they already match).

## Outcome
Excel's MULTINOMIAL is identified as a log10 running sum (fit, decision 0056 style). One double accumulator adds
the correctly rounded `log10(k)` for k = 1..sum, then subtracts `log10(k)` for k = 1..n for each argument,
taking the last argument first. The result is `exp(acc * ln 10)` (`multinomial_value`,
functions/math_and_trigonometry/multinomial.rs). The table `LOG10_OF` keeps the terms correctly rounded on
every platform; glibc's `log10` is one ulp off at 10 of them. Numeric text is now its number, both typed and
in a range cell (`numeric_text_as_number`).
New probe probes/multinomial-algorithm-sweep (70 cells, excel-graph 2026-10-15). It covers 26 single
arguments, argument order, pairs, triples, and COMBIN for comparison.
Fit scores, bit-exact out of 72 MULTINOMIAL cells across the three probe cases:
- chosen: 64; the 8 misses are single arguments, each one ulp off
- `exp(GAMMALN(s+1))` over exact factorials: 17
- natural-log sums: at most 16
- `10^x` final step: 30
- arguments in their given order: 60
- glibc `log10`: 57
- `ln(k)/ln 10` or `ln(k)*log10(e)` terms: at most 33
- x87 80-bit terms or accumulator: at most 54
- greedy one-ulp nudges of any single term: no gain
Excel's COMBIN is inexact as well, with rounding that differs from MULTINOMIAL's. It is recorded in
numbers.md "COMBIN is inexact" and filed as **T-1598**.
Eval delta:
- ironcalc-calc fixed 6 (MULTINOMIAL.xlsx A1, A8, B3, B16, B22, D12)
- probes fixed 14 (math-argument-modes C9, C35, C47, C49:C53; multinomial-large-arguments A1, A7, A11, A13, A14, A20)
- the new case adds 15 known failures: the 8 one-ulp MULTINOMIAL cells and 7 COMBIN cells (T-1598)
- 0 regressions; poi unchanged; gen-m5..m9 MULTINOMIAL cells unchanged
