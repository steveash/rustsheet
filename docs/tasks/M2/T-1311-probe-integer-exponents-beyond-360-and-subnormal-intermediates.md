---
id: T-1311
title: "Probe integer exponents beyond 360 and subnormal intermediates for the power operator"
milestone: M2
lane: harness
status: done
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
- [x] Probe case with oracle values; numbers.md updated; engine adjusted if Excel differs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Four probe cases (`probes-numeric/power-large-integer-exponents{,-2,-3,-4}`, about 450 oracle cells)
showed Excel differs from the T-1302 rule in three places, now implemented: subnormal results
flush to 0, a negative base with |n| >= 2^32 - 1 is `#NUM!`, and base 10 with |n| >= 23 is the
correctly rounded `1En` (or its reciprocal). See numbers.md "Integer exponents beyond 360".
All cells pass except the three non-integer `10^x` cells, which stay as known failures (follow-up T-1319).
