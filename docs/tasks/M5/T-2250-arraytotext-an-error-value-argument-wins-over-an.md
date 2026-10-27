---
id: T-2250
title: "ARRAYTOTEXT: an error value argument wins over an invalid format (#NUM! before #VALUE!)"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`ARRAYTOTEXT(<error>, 2)` is the argument's error (`#NUM!`), not `#VALUE!` for the invalid format 2.

## Context
Left over from T-2169: gen-m5/4febabc66cfdf9a1 Calc!A156 `=ARRAYTOTEXT(SINH(134217727),2)` is `#NUM!` in Excel, ours `#VALUE!` (`make eval-show CASE=gen-m5/4febabc66cfdf9a1.case.json`). Probe whether the error argument wins over format validation generally (excel-probe skill) before implementing.

## Acceptance criteria
- [ ] The cell leaves eval/baseline/gen-m5.txt (`make eval-baseline`), no new known failure.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
