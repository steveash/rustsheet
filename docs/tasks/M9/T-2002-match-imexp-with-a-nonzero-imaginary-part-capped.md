---
id: T-2002
title: "Match IMEXP with a nonzero imaginary part capped at DBL_MAX (T-1827 A12)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IMEXP with a nonzero imaginary part and an overflowing real part caps the modulus at DBL_MAX as Excel does.

## Context
Known failure probes/eng-im-overflow S!A12 (`IMEXP("1000+1i")`); ours gives `#NUM!`. Probe the boundary first (`IMEXP("710+1E-300i")`, `IMEXP("1000-1i")`, `IMEXP("1000+3.14159i")`); same mechanism as T-1950. TODO(T-2002) in `imexp`.

## Acceptance criteria
- [ ] A12 passes and the boundary cases are pinned by a probe; IMEXP of a pure real stays `#NUM!`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
