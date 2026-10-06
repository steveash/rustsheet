---
id: T-0243
title: "CONCAT and TEXTJOIN give #CALC! past 32767 characters"
milestone: M2
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
CONCAT and TEXTJOIN return `#CALC!` when the joined text exceeds 32,767 characters, as Excel does.

## Context
probes/text-result-limit C8 (`LEN(CONCAT(REPT("a",32767),"b"))`) and C9 (TEXTJOIN) are `#CALC!` in excel-graph (web only; confirm on excel-desktop if available, decision 0019). See docs/excel-semantics/text.md "SUBSTITUTE result longer than 32,767 characters". Find the existing implementations with `rg fn_concat\b fn_textjoin`; size the result before building it (directive 5).

## Acceptance criteria
- [ ] probes/text-result-limit C8 and C9 leave eval/baseline/probes.txt with no new F lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
