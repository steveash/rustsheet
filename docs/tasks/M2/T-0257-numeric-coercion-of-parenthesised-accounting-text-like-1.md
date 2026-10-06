---
id: T-0257
title: "Numeric coercion of parenthesised accounting text like (1) = -1"
milestone: M2
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Number text in parentheses is read as a negative number everywhere Excel coerces text to a number: `0+"(1)"` is -1 and `COUNT("(1)","x")` is 1.

## Context
Found in T-0252. Evidence: probes/paren-negative-text (excel-graph): `=0+"(1)"` = -1, `=SUM("(1)")` = -1, `=0+"(1,000)"` = -1000, `=0+"-(1)"` is #VALUE!, `COUNT("(1)","x")` = 1. `cast_number` (crates/rustsheet-engine/src/cast.rs) does not read the form, so these cells fail. See docs/excel-semantics/coercion.md "Direct text arguments of COUNT".

## Acceptance criteria
- [ ] Every cell of probes/paren-negative-text passes and leaves eval/baseline/probes.txt; no new baseline lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
