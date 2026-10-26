---
id: T-2696
title: "Probe where POWER's overflowing y*ln x rule switches on (bases 0.15-0.2, exponents 1E+307-1.7E+308)"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Pin down the edge of numbers.md "POWER when y·ln x overflows" (T-2661): the engine's test is "the double product
`|y|·ln x` is infinite", and the probe evidence only brackets it coarsely.

## Context
- T-2661's probes (power-huge-exponent, power-op-huge-exponent) jump from base 0.1 (product overflows, `#NUM!` / 0)
  to 0.5 (finite product, 0 / `#DIV/0!`), and from exponent 9.99999999999999E+307 to 1E+300. The predicted switch for
  y = 9.99999999999999E+307 is at ln x = -1.797..., x about 0.166; other rules (a fixed exponent cutoff with |ln x| > 1)
  fit the same cells.
- Code: `positive_power` in crates/rustsheet-engine/src/arithmetic.rs. Its test uses a once-rounded `y * ln_x`, while
  the value path uses `extended_product`; they differ only at the boundary.

## Acceptance criteria
- [ ] A probes case (tag M5, excel-graph values): POWER and `^` for bases 0.15, 0.16, 0.165, 0.166, 0.167, 0.17, 0.2,
      and for 0.5 and 0.9 with exponents 1E+307, 5E+307, 1.5E+308, 1.7E+308 (and their negatives; `1E308` literals cannot be
      filled by excel-graph, so build big exponents as products such as `1.7E+307*10`).
- [ ] The engine matches every cell (adjust `positive_power` if the edge differs), and numbers.md records the edge.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Integer exponents up to 2^53 (the square-and-multiply path).
