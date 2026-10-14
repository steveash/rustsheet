---
id: T-1288
title: "Probe VALUETOTEXT with an error value and an invalid or error format, and a range-reference format"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
VALUETOTEXT matches Excel when the value is an error and the format is invalid or an error, and when the format is a range reference.

## Context
T-1229 left these unprobed (fn_valuetotext in functions/text/common.rs): VALUETOTEXT(#N/A,2), VALUETOTEXT(#N/A,#DIV/0!), VALUETOTEXT(A1,B1:B2). See docs/excel-semantics/text.md. Use the excel-probe skill.

## Acceptance criteria
- [ ] Probe cells for the three forms have oracle values, the engine matches them, and text.md drops the UNVERIFIED note.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
