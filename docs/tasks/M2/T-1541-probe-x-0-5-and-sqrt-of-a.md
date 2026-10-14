---
id: T-1541
title: "Probe x^-0.5 and SQRT of a subnormal where rounding the root first differs"
milestone: M2
lane: functions-math
status: todo
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
- [ ] A probes-numeric case with operands that separate the hypotheses, with oracle values.
- [ ] The engine matches it; numbers.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
