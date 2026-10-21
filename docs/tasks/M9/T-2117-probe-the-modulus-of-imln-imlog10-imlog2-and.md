---
id: T-2117
title: "Probe the modulus of IMLN, IMLOG10, IMLOG2 and IMSQRT and use the hypot modulus where Excel does"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
IMLN, IMLOG10, IMLOG2 and IMSQRT compute `|z|` as Excel does, with one modulus definition in complex.rs.

## Context
T-1950 found Excel's IMABS and IMPOWER modulus is `hypot`, not `sqrt(x²+y²)`
(`IMABS("0.7071067811865476+0.7071067811865476i")` = 0.9999999999999993, probes/eng-impower-past-cut A15) and added
`modulus` in functions/engineering/complex.rs. `imln`, `imlog10`, `imlog2` and `imsqrt` still use `sqrt(x*x+y*y)`;
their Excel modulus is unprobed. Probe each on bases where the two differ in the last bit (the 16-digit 1/√2 literal,
`3E200+4E200i` where `x*x` overflows, `3E-200+4E-200i` where it underflows), then switch the ones that match.

## Acceptance criteria
- [ ] An oracle-filled probe covers the four functions on those bases; each uses `modulus` where Excel matches it,
  and engineering.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMPOWER and IMABS (done in T-1950).
