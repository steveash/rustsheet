---
id: T-1569
title: "Probe ATANH(1) and ATANH(-1) (infinite results) against Excel"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
ATANH(1) and ATANH(-1) give the same value or error as Excel, probed with the oracle.

## Context
Split from T-1202: `f64::atanh` returns an infinity there, not NaN, so the NaN-to-`#NUM!` mapping in `single_number_fn!` does not catch it. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes with oracle values; the engine matches (or a fix lands).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
