---
id: T-1635
title: "Probe UNICODE of a lone high surrogate followed by another character"
milestone: M2
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish with the Excel oracle what UNICODE returns for a lone high surrogate followed by another character, and make our result match.

## Context
T-1581 made `fn_unicode` return `#VALUE!` for `UNICODE(MID("a😀b",2,2))` (lone high surrogate then `b`) by inference from probes/text-lone-surrogates A3. Unprobed; see docs/excel-semantics/text.md "MID and REPLACE positions…". Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case holds the Excel value for UNICODE(MID("a😀b",2,2)) and the text.md note cites it.
- [ ] Our result matches (fix `fn_unicode` if not).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
