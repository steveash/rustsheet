---
id: T-0141
title: "MOD returns #NUM! for a huge quotient: probe the threshold and match it"
milestone: M1
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
MOD returns `#NUM!` exactly where Excel does when number/divisor is huge.

## Context
Found by T-0106: `=MOD(999999999999957+0.5,1)` is `#NUM!` in Excel (probes/round-family A20,
oracle excel-graph 2026-10-05); RustSheet returns 0.5. The threshold on the quotient is unknown
(older Excel documentation mentions 2^27 = 134217728; not verified). See
docs/excel-semantics/numbers.md "MOD is #NUM! for a huge quotient". Code:
`fn_mod` in crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) brackets the threshold: quotients around 2^27, 2^31, 2^48,
      2^53 and 1E+15, both signs of number and divisor, fractional divisors.
- [ ] `fn_mod` returns `#NUM!` past the measured threshold; probes/round-family A20 passes; the
      ironcalc-calc MOD_QUOTIENT cells still pass.
- [ ] numbers.md entry updated to VERIFIED with the threshold.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
