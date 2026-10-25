---
id: T-2332
title: "Probe TEXT on number-like text with a text section, date/percent text, and other control characters"
milestone: M5
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the TEXT choices T-2158 made without it, so each is confirmed or corrected.

## Context
T-2158 coerces number-like text in `fn_text` (functions/text/common.rs) and prints control characters as literals (formatter/lexer.rs). UNVERIFIED there: text with a text section (`has_text_section`, which also counts `@` and `;` inside quotes), date and percent text, VALUE's space-only trim, other control characters. See docs/excel-semantics/number-formats.md (T-2158 entry); use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probes case covers `TEXT("001","@")`, `TEXT("1","0;0;0;@")`, `TEXT("1","0;0")`, `TEXT("50%","0.00")`, `TEXT("1/2/2020","0")`, `TEXT(" 1 ","0")`, `TEXT("1 ","0")`, `"\"@\"0"`, `\@`, and CHAR(9), CHAR(13), CHAR(127) in the code, with oracle values.
- [ ] number-formats.md and the engine match the values; mismatches are fixed or filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/text-number-like-text-and-control-chars (29 cells, excel-graph). Number-like text is always coerced to its number whatever the code's sections (the engine's `has_text_section` guard was wrong for 4 cells: A1, A9, A10, A14); removed it in `fn_text`. Everything else T-2158 chose (date and percent text, space trim, control characters) matched. Eval delta: the new probe case passes 29/29 (its baseline line is the `C` pass record, no known failures), 0 regressions. Unprobed: a tab around the number text.
