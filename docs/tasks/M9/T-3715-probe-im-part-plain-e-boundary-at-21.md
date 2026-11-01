---
id: T-3715
title: "Probe IM* part plain/E boundary at 21 characters for exponents -9 to -12"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Settle whether an IM* part of exactly 21 plain characters is plain at exponents -9 to -12, and correct `value_to_text` if not.

## Context
T-3560 evidenced the 21-character limit at exponents -6 to -8 only (probes/eng-im-part-plain-boundary) and applies it to every negative exponent. Code: number_format.rs `MAX_COMPLEX_FRACTION_PLAIN_LEN`; semantics: engineering.md "An IM* part with 14 significant digits at exponent -6".

## Acceptance criteria
- [ ] A probe with 9-11 digit parts at exponents -9 to -12 that make exactly 21 characters is oracle-filled.
- [ ] The code and engineering.md match the probe.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
