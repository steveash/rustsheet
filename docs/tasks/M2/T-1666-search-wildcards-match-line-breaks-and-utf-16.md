---
id: T-1666
title: "SEARCH wildcards match line breaks and UTF-16 units like criteria (probe first)"
milestone: M2
lane: functions-text
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`SEARCH` with `?`/`*` treats line breaks and surrogate halves as Excel does.

## Context
T-1642 moved criteria and wildcard MATCH/XMATCH to `WildcardPattern` (functions/criteria.rs), which matches `?` and `*` over UTF-16 units, line breaks included (docs/excel-semantics/criteria.md "Wildcards match line breaks"). `SEARCH` (functions/text/position.rs) still builds a regex with `from_wildcard_to_regex`, whose `.` skips `\n` and reads a pair as one character. Probe first (excel-probe skill): e.g. `SEARCH("a?b","x"&"a"&CHAR(10)&"b")`, `SEARCH("a*b","a"&CHAR(10)&"b")`, `SEARCH("?b","😀b")`. If Excel agrees with criteria, extend `WildcardPattern` with an unanchored search that returns the start unit and retire `from_wildcard_to_regex`.

## Acceptance criteria
- [ ] A probe case records Excel's SEARCH results for wildcards over line breaks and a surrogate pair; the engine matches it.
- [ ] One wildcard implementation remains (no regex translation), unless the probe shows SEARCH differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
