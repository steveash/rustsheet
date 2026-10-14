---
id: T-1495
title: "Integer powers: round each square-and-multiply product twice, like Excel's `*`"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Integer powers use square-and-multiply with twice-rounded products (64 bits, then double; Excel's `*`,
`arithmetic::extended_product`), so `SQRT(5)^n`, `(0.2^-0.5)^n` and `POWER(SQRT(5),n)` give Excel's
doubles, as T-1524 found.

## Context
Kernel map (T-1522, decision 0056): rank 13 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
T-0221 probe probes-numeric/power-integer-full-mantissa (excel-graph, 2026-10-13): right-to-left
square-and-multiply (numbers.md "Integer exponents") matches every cell of 7 full-significand
bases (pi, e, 1.1, 1/3, 0.7, SQRT(2), 1.0000001) at 19 exponents 3..255, but only 2 of the 38
cells whose base is 2.23606797749979 (SQRT(5)). Excel's values track `exp(n·ln_double(b))` (10 of
19) and drift up to 2 ulps below it by n = 255; the models already rejected are listed in
numbers.md under "Integer exponents beyond 360" ("Unresolved (T-1495)"). Questions to probe: is it
the base value (try the neighbors of SQRT(5), SQRT(3), SQRT(6), SQRT(7), SQRT(10), other bases
whose square is one ulp off an integer) or the exponent? (Answered by T-1524, below: neither; it is
how the products round.) Known failure: gen-m2/fa437879253827be
Calc!A23 (`(0.2^-0.5)^99`). Code: `integer_power` in crates/rustsheet-engine/src/arithmetic.rs.

**Resolved model (T-1524, fit lab `power-sqrt5`, 2026-10-14):** Excel's square-and-multiply rounds each
product twice, like its `*` operator (64 bits, then double; numbers.md "Division rounds twice"). With that,
the loop gives all 20 `SQRT(5)^n` cells (n = 3..255 and -99); with products rounded once (the engine's
`square_and_multiply` in arithmetic.rs uses plain `*`) it gives 1. The double rounding matches exactly the
same cells as today on the other bases and exponents (fits `power-mantissa` 76/76, `power-1+r` 462/462,
`power-large-int`), so the fix is to use `arithmetic::extended_product` for the loop's products. Recorded in
numbers.md "Integer exponents beyond 360" (the former "Unresolved (T-1495)" paragraph). All 42 cells with
this base match with twice-rounded products (2 with products rounded once); the fit lab scores the 20
`SQRT(5)^n` cells (`power-sqrt5`), and the 22 `(0.2^-0.5)^n`, A181 and A182 cells were checked outside it.

## Acceptance criteria
- [ ] `square_and_multiply` in crates/rustsheet-engine/src/arithmetic.rs rounds each product with
      `extended_product`, and the numbers.md entry's "Implemented in" says so.
- [ ] All 42 cells of `probes-numeric/power-integer-full-mantissa` whose base is 2.23606797749979 pass:
      `SQRT(5)^n` (20 cells), `(0.2^-0.5)^n` (20), A181 `(SUM(0,(0.2^(-0.5)))^99)` and A182
      `POWER(SQRT(5),99)`.
- [ ] gen-m2/fa437879253827be Calc!A23 (`(0.2^-0.5)^99`) leaves `eval/baseline/gen-m2.txt`.
- [ ] Unchanged elsewhere: the fit lab's `power-mantissa` (76/76), `power-1+r` (462/462) and
      `power-large-int` (the same cells) scores, and no new lines in any `eval/baseline/` file.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Operations other than integer powers.
