---
id: T-2141
title: "Probe ACOSH near x=20.83 and x=180.08: the two one-ulp misses of excel_ln"
milestone: M9
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
defer: phase-1b
---

## Goal
Know why `excel_ln`-based ACOSH misses Excel by one ulp at x = 20.82932287508296 and 180.0774330018289, and match it.

## Context
probes-numeric/t2133-asinh-acosh-ln Sheet1!B167 and B226 (T-2133). Candidates: the argument `x + sqrt(x*x-1)` kept in an 80-bit register into FYL2X, or an FYL2X near-tie (Excel's own results vary there, probes/ln-fyl2x-kernel). `sqrt((x-1)(x+1))` and an fma `x*x-1` change nothing. See numbers.md "Inverse hyperbolics".

## Acceptance criteria
- [ ] A probe around the two points with excel-graph values; ACOSH follows the evidence or numbers.md records the near-tie.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
