---
id: T-1000
title: "Probe number-to-text carry at exponent magnitude 99 (9.99999999999995E+99, E-100)"
milestone: M1
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's number text is known for a rounding carry at exponent magnitude 99, and the engine matches it.

## Context
T-0199 rounds the 15-digit mantissa half up to 14 digits at |exp| >= 99; a carry to 10^14 raises the exponent (e.g. `9.99999999999995E+99` -> "1E+100", `9.99999999999995E-100` -> "1E-99", `9.99999999999995E-99` -> "1E-98") without a probe. See docs/excel-semantics/numbers.md "Exponent width"; code: `number_to_text` in number_format.rs. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe case with oracle values for the three carry inputs above; numbers.md updated; engine fixed if it differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
