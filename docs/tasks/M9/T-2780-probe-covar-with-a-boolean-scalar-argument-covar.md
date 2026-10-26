---
id: T-2780
title: "Probe COVAR with a boolean scalar argument (COVAR({1,2,3},TRUE), COVAR(-2.5,FALSE)); T-2575 moved it with COVARIANCE.P unprobed"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
COVAR's behavior for a typed or computed boolean scalar argument is established by an Excel probe, and the engine matches it.

## Context
T-2575 made COVARIANCE.P/.S read a boolean scalar as one value; COVAR shares `fn_covariance_p` (`Function::CovarCompat`) and moved with it, unprobed. See statistics.md "COVARIANCE: a boolean scalar is one value (T-2436)" (UNVERIFIED line). Code: functions/statistical/paired.rs `sample_arg`.

## Acceptance criteria
- [ ] A probe case with oracle values for `COVAR({1,2,3},TRUE)`, `COVAR(-2.5,FALSE)` and `COVAR(B1,FALSE)` is gated; the engine matches, or the deviation is fixed.
- [ ] The statistics.md entry's UNVERIFIED line is resolved.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
