---
id: T-2683
title: "TEXT with an elapsed token and AM/PM ([h]:mm AM/PM) is #VALUE!"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(x,"[h]:mm AM/PM")` is #VALUE! as in Excel, instead of a clock with "AM".

## Context
Found by T-2385: probes/elapsed-no-seconds-boundary A31 `=TEXT((3-0.6/3600)/24,"[h]:mm AM/PM")` is #VALUE! in Excel
(excel-graph); ours "2:59 AM". docs/excel-semantics/number-formats.md "Elapsed-time tokens". The date arm is in
crates/rustsheet-engine/src/formatter/format.rs (`ParsePart::Date`). Probe whether `[m]`/`[s]` with AM/PM and `A/P` do the same
before generalizing (excel-probe skill), and whether the number-format cell display matches (the probe only covers TEXT).

## Acceptance criteria
- [ ] probes/elapsed-no-seconds-boundary A31 passes; the rule for `[m]`/`[s]` and `A/P` is probed and recorded in number-formats.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Done by T-1366: a bracketed elapsed token before AM/PM or A/P is #VALUE! (elapsed-no-seconds-boundary A31 now passes).
