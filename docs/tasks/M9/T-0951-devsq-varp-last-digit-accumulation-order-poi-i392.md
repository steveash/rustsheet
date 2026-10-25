---
id: T-0951
title: "DEVSQ/VARP last-digit accumulation order (poi I392, K1456, E392, F392, J392)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
DEVSQ and VARP reproduce Excel's last digit on poi EverythingTests!I392, K1456, E392, F392 and J392.

## Context
Left over from T-0945. `DEVSQ("1","2",2)` expects 0.6666666666666666 (engine 0.6666666666666661 with sumsq - sum^2/n; a two-pass mean/deviation sum gives the expected value, but VARP(TRUE,FALSE,TRUE) expects 0.2222222222222222 and two-pass gives ...24). Numeric fitting: find the accumulation formula that satisfies all five cells and the currently passing ones.

## Acceptance criteria
- [ ] The five cells leave eval/baseline/poi.txt with no new F lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).
