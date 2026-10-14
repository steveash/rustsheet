---
id: T-1541
title: "Probe x^-0.5 and SQRT of a subnormal where rounding the root first differs"
milestone: M2
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's `x^-0.5` and SQRT of a subnormal are probed, and the engine matches them.

## Context
T-1500 made SQRT and `^0.5` round twice (`arithmetic::extended_sqrt`). Not probed: `x^-0.5` (engine: 1 / plain `sqrt`, divided with `extended_quotient`). Three hypotheses differ on some operands: plain sqrt then double-rounded division; double-rounded sqrt then double-rounded division; the 64-bit root then one division. The 6 `^-0.5` cells of probes-numeric/power-reciprocal-rounding do not separate them. A positive subnormal radicand also takes the plain root today. Pick operands by exact simulation, probe via the `excel-probe` skill, record in docs/excel-semantics/numbers.md.

## Acceptance criteria
- [x] A probes-numeric case with operands that separate the hypotheses, with oracle values.
- [x] The engine matches it; numbers.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
probes-numeric/power-neg-half-rounding (40 cells, excel-graph) separates the hypotheses. Excel's
`x^-0.5`, `POWER(x,-0.5)` and `1/SQRT(x)` all divide 1 by the twice-rounded root (as SQRT) with a
twice-rounded division (hypothesis 2) on 4 operands where all three differ. A subnormal radicand,
kept by `-`, also rounds its root twice. `non_integer_power` now uses `extended_sqrt` for -0.5, and
`significand_and_exponent` normalizes a subnormal so `extended_sqrt` covers it. The case passes 40/40
(20/40 before, by the simulated old paths; probes-numeric has no baseline, so a unit test in test_fn_power.rs guards it). Probe note: a 16-digit integer literal loses
its last digit in Excel, so the operands have at most 15 digits.

