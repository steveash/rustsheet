---
id: T-1574
title: "Probe GCD/LCM/MULTINOMIAL precedence of a negative against typed text, errors and LCM ranges"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish with the Excel oracle which of a negative number's `#NUM!` and a later type error or error value wins in GCD, LCM and MULTINOMIAL, for the paths T-1261 left unprobed, and make the engine match.

## Context
T-1261 defers a negative's `#NUM!` only past text or boolean cells of a range argument (evidenced by gen-m6/gen-m7 cells). Unprobed, still the immediate `#NUM!` in the engine (docs/excel-semantics/coercion.md "GCD / LCM / MULTINOMIAL range cells"):
`GCD(-1,"a")`, `GCD(-1,TRUE)`, `GCD(-1,NA())`, `GCD(NA(),-1)`, `GCD(-1,)`, a range error cell after a negative, an array-literal element after a negative, `LCM(-1,<range with TRUE>)`, `LCM(-1,"a")`, `MULTINOMIAL(-1,<range with text>)`, `MULTINOMIAL(-1,"a")`, `MULTINOMIAL(-1,NA())`. Code: gcd_lcm.rs and multinomial.rs under crates/rustsheet-engine/src/functions/math_and_trigonometry/.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes covers the cells above, filled by the oracle (excel-probe skill).
- [ ] The engine matches the oracle values; coercion.md records the full precedence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MULTINOMIAL's inexact values (T-1190).
