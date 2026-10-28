---
id: T-2661
title: "POWER of a tiny base to a huge exponent: Excel's #NUM! and 0 where ours gives 0 and #DIV/0! (calipers formulas_math_basic)"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
POWER (and `^`) of a tiny base to a huge exponent give Excel's error or value.

## Context
Phase 1 assessment, 2026-10-25 (decision 0114). In `calipers/roundtrip/formulas_math_basic` (sheet
`Math Tests`) and its twin `formulas_datetime` (sheet `Sheet2`), C10 is `10^(-307)`, C12 is
`9.99999999999999E+307` and C13 is `-9.99999999999999E+307`:
- J75 `POWER($C$10,$C$12)`: Excel `#NUM!`, ours 0;
- K75 `POWER($C$10,$C$13)`: Excel 0, ours `#DIV/0!`.

The same sheets' `MOD($C$6,$C$13)` (Excel 0, ours -9.99999999999999E+307) and `MOD($C$7,$C$12)`
(Excel 0, ours 9.99999999999999E+307) are T-2654's rule; name them in T-2654 rather than here.

The goldens are desktop evidence, so probe first (`excel-probe` skill, excel-graph): underflow and
overflow of POWER and `^` for a base below 1 with a huge positive and negative exponent, a base
above 1 likewise, and 0 and 1 as bases. Record the rule in docs/excel-semantics/numbers.md.

## Acceptance criteria
- [x] A probes case (milestone tag M5) with excel-graph values; the rule is in numbers.md.
- [x] Ours matches the probe and the calipers J75/K75 cells in both workbooks.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns is within half a unit of the 15th significant digit of Excel's value, or
has the right type, error and text (decision 0083). Bit-level residue goes to a `defer: phase-1b`
follow-up (decision 0114).

## Out of scope
- MOD (T-2654).

## Outcome
- Probes `probes/power-huge-exponent` (POWER, M5) and `power-op-huge-exponent` (`^`, M2), 15 bases by 20 exponents
  each, oracle excel-graph 2026-10-25. The only cells that differed: for a base below 1 whose `|y|·ln x` overflows
  (bases 1E-307, 1E-300, 1E-10, 0.1 to ±9.99999999999999E+307), Excel treats the infinite product as an overflow:
  `#NUM!` for the positive exponent, 0 for the negative one. A finite product that underflows (0.5, 0.9, any base to
  ±1E+300) keeps 0 / `#DIV/0!`.
- `positive_power` in arithmetic.rs maps an infinite product to an overflow (0 keeps ln 0 = -∞). numbers.md
  "POWER when y·ln x overflows" is VERIFIED.
- Eval: two new probe cases at 600/600; no regressions. Calipers J75/K75 (`POWER(10^-307,±9.99999999999999E+307)`)
  were not re-run locally (suite not fetched); probe cells A1-A2 have the same operands, and the engine now matches them.
