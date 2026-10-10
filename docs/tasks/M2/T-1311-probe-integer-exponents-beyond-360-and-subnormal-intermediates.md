---
id: T-1311
title: "Probe integer exponents beyond 360 and subnormal intermediates for the power operator"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for `x^n` and `POWER(x,n)` with integer |n| > 360 and with subnormal intermediate results.

## Context
T-1302 applies right-to-left square-and-multiply to every finite integer exponent up to 2^53, but the oracle only covers 1..360. Probe n in {1000, 1E6, 2^31, 2^53} with x near 1, negative bases at huge even and odd n, and negative n with a subnormal reciprocal. See numbers.md "Integer exponents"; code: `integer_power` in arithmetic.rs. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe case with oracle values; numbers.md updated; engine adjusted if Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
