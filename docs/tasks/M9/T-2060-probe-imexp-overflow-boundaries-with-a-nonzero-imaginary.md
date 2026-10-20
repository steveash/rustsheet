---
id: T-2060
title: "Probe IMEXP overflow boundaries with a nonzero imaginary part (T-2002)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IMEXP overflow with a nonzero imaginary part is pinned by probes beyond A12.

## Context
T-2002 caps the modulus at DBL_MAX for any nonzero finite imaginary part from A12 alone. Probe `IMEXP("710+1E-300i")`, `IMEXP("1000-1i")`, `IMEXP("1000+3.14159i")` `IMEXP("1000+1E-300i")` and `IMEXP("709.9+1i")` (overflow window where exp(x)·cos y is finite) and fix `imexp` to match.

## Acceptance criteria
- [ ] Probe cases recorded with oracle values; `imexp` matches them; engineering.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 31 IMEXP cases (probes/eng-imexp-overflow-imag, excel-graph). The DBL_MAX cap holds for negative and
positive imaginary parts and `709.9+1i`; an imaginary part with |y| < sqrt(DBL_EPSILON) (boundary 1.4E-8 error, 1.5E-8
capped) is `#NUM!`. `imexp` matches; the IM* text writer no longer overflows a part near DBL_MAX. Eval: 30 of 31
cells pass; A3 differs in the last digits of a large-sine imaginary part (baselined). Unprobed: the same rule near y = pi.
