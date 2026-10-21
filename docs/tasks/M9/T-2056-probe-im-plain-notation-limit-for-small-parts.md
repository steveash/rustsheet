---
id: T-2056
title: "Probe IM* plain-notation limit for small parts (exponents -6 to -15)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
IM* small-part text switches from plain to E notation at the exponent Excel uses, evidenced by probes.

## Context
T-2001 writes parts with exponent -5 to -1 plain past 20 characters (a printf %G guess bounded only by eng-im-overflow A42 and eng-impower A9). Code: `complex_part_to_text` in number_format.rs; notes in engineering.md "IM* results that overflow".

## Acceptance criteria
- [x] Oracle values for IM* parts at exponents -6 to -15 (15-digit mantissas) are in a probe case; `MIN_COMPLEX_PLAIN_EXPONENT` matches them; the semantics note cites them.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed exponents -1 to -16 (real and imaginary parts, 15-digit mantissa) in probes/eng-im-small-part: -5 is the last plain exponent, -6 on is E notation, so `MIN_COMPLEX_PLAIN_EXPONENT = -5` was already right. Comments and engineering.md now cite the probe. No eval delta in code.
