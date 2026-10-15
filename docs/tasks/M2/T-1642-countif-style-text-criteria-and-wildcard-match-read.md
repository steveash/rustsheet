---
id: T-1642
title: "COUNTIF-style text criteria and wildcard MATCH read lone surrogates exactly (probe first)"
milestone: M2
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`COUNTIF`-style `=`, `<>`, `<`, `>` criteria and wildcard patterns (`*`, `?`) that contain or meet a lone surrogate match as Excel does.

## Context
T-1585 made `compare_values`, EXACT and exact MATCH lone-surrogate-exact, but the criteria matchers in functions/criteria.rs still call `cmp_text_excel(&str)` (lossy U+FFFD), and `wildcard_text_matcher_for` only handles a lone-half pattern with no wildcards. Excel's behavior here is unprobed (docs/excel-semantics/text.md, lone surrogates entry): write a probe first (excel-probe skill).

## Acceptance criteria
- [ ] A probe case records Excel's COUNTIF/SUMIF and wildcard-MATCH results for a lone half; the engine matches it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
