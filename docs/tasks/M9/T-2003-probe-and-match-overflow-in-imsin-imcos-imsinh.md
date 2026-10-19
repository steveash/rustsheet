---
id: T-2003
title: "Probe and match overflow in IMSIN IMCOS IMSINH IMCOSH and IMPRODUCT with a blank"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IMSIN, IMCOS, IMSINH, IMCOSH and IMPRODUCT-with-a-blank-cell overflow give what Excel gives, not `inf` text.

## Context
Found by T-1827 review: `IMCOSH("1000")`, `IMSIN("1+1000i")` print inf text; `IMPRODUCT("1E200","1E200",<blank>)` gives 0 via the blank shortcut. Also probe IMSUB overflow, `IMSUM("9E307","9E307","-9E307")` and an overflow followed by a later error argument. Probe first (excel-probe skill).

## Acceptance criteria
- [ ] A probe pins each form and ours matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
