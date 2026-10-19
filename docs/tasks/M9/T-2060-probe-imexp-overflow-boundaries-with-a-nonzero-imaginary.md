---
id: T-2060
title: "Probe IMEXP overflow boundaries with a nonzero imaginary part (T-2002)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IMEXP overflow with a nonzero imaginary part is pinned by probes beyond A12.

## Context
T-2002 caps the modulus at DBL_MAX for any nonzero finite imaginary part from A12 alone. Probe `IMEXP("710+1E-300i")`, `IMEXP("1000-1i")`, `IMEXP("1000+3.14159i")` and `IMEXP("1000+1E-300i")` and fix `imexp` to match.

## Acceptance criteria
- [ ] Probe cases recorded with oracle values; `imexp` matches them; engineering.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
