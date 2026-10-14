---
id: T-1495
title: "Fit Excel's integer powers of SQRT(5): not square-and-multiply"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`SQRT(5)^n` (and `(0.2^-0.5)^n`, `POWER(SQRT(5),n)`) gives Excel's double for integer n.

## Context
Kernel map (T-1522, decision 0056): rank 13 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
T-0221 probe probes-numeric/power-integer-full-mantissa (excel-graph, 2026-10-13): right-to-left
square-and-multiply (numbers.md "Integer exponents") matches every cell of 7 full-significand
bases (pi, e, 1.1, 1/3, 0.7, SQRT(2), 1.0000001) at 19 exponents 3..255, but only 2 of the 38
cells whose base is 2.23606797749979 (SQRT(5)). Excel's values track `exp(n·ln_double(b))` (10 of
19) and drift up to 2 ulps below it by n = 255; the models already rejected are listed in
numbers.md under "Integer exponents beyond 360" ("Unresolved (T-1495)"). Questions to probe: is it
the base value (try the neighbors of SQRT(5), SQRT(3), SQRT(6), SQRT(7), SQRT(10), other bases
whose square is one ulp off an integer) or the exponent? Known failure: gen-m2/fa437879253827be
Calc!A23 (`(0.2^-0.5)^99`). Code: `integer_power` in crates/rustsheet-engine/src/arithmetic.rs.

## Acceptance criteria
- [ ] A probe sweep separates the candidate rules; numbers.md entry updated.
- [ ] power-integer-full-mantissa passes and gen-m2/fa437879253827be Calc!A23 leaves the baseline,
  with no new failures in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Operations other than integer powers.
