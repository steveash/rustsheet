---
id: T-2661
title: "POWER of a tiny base to a huge exponent: Excel's #NUM! and 0 where ours gives 0 and #DIV/0! (calipers formulas_math_basic)"
milestone: M5
lane: functions-math
status: in-progress
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
- [ ] A probes case (milestone tag M5) with excel-graph values; the rule is in numbers.md.
- [ ] Ours matches the probe and the calipers J75/K75 cells in both workbooks.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns is within half a unit of the 15th significant digit of Excel's value, or
has the right type, error and text (decision 0083). Bit-level residue goes to a `defer: phase-1b`
follow-up (decision 0114).

## Out of scope
- MOD (T-2654).
