---
id: T-1635
title: "Probe UNICODE of a lone high surrogate followed by another character"
milestone: M2
lane: functions-text
status: done
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
- [x] A probe case holds the Excel value for UNICODE(MID("a😀b",2,2)) and the text.md note cites it.
- [x] Our result matches (fix `fn_unicode` if not).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/text-unicode-lone-surrogate-followed (10 cells, excel-graph values). Excel: `UNICODE(H&"x")` is `#VALUE!` and `UNICODE(L&"x")` is 56832, so the T-1581 inference held; the task's example `UNICODE(MID("a😀b",2,2))` is the whole pair (128512), not a lone unit. Ours already matched all 10 cells (no code change). text.md updated. Eval delta: new case, 10/10 pass.
