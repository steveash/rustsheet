---
id: T-2498
title: "Probe TEXTBEFORE and TEXTAFTER match_end 1.5, 2, -1 and TRUE"
milestone: M7
lane: functions-text
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Establish with the Excel oracle what TEXTBEFORE and TEXTAFTER do for match_end 1.5, 2, -1 and TRUE, and record it.

## Context
T-2412 cut match_end toward zero (-0.0001 and 0.5 are 0, text.md "TEXTBEFORE and TEXTAFTER: instance_num past the text"). That makes 1.5 read as 1 in ours, where it was `#VALUE!`; 1.5, 2, -1 and TRUE are unprobed. Use the excel-probe skill on `TEXTAFTER("a","zz",1,0,<value>)`. Code: `around_options` in functions/text/before_after.rs.

## Acceptance criteria
- [ ] A probe case with oracle values for each value, the text.md entry updated, and ours changed if it differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
