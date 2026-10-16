---
id: T-1656
title: "TEXT of text with a format holding @ and a date token (x@y) is #VALUE!"
milestone: M2
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
TEXT of a text value with a one-section format holding `@` and a date token is `#VALUE!`, as in Excel.

## Context
probes/text-lone-surrogates A91 (`LEN(TEXT("😀","x@y"))`) is `#VALUE!` in Excel (excel-graph); the engine returns 3. A85, A89, A90, A92, A94 fail the same way. Valid quoted formats (`"x"@"y"`, A97-A101) already pass. Probe first which tokens beside `@` error (`y`, `d`, `m`, `h`, `s`, `b`, `e`, `g`) for text values; see docs/excel-semantics/text.md (lone surrogates) and number-formats.md.

## Acceptance criteria
- [ ] Probe cells for the date-token set beside `@` on text, with oracle values; the engine matches them and A85, A89-A92, A94 pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Done: probe `probes/text-at-with-tokens` (89 cells, oracle filled) and the rule in docs/excel-semantics/number-formats.md ("TEXT of a text value: `@` with other tokens in one section"). Ours passes 16 of 89 (`rustsheet-eval run --suite probes --filter text-at-with-tokens`, then `show probes/text-at-with-tokens.case.json`).
Not done: the engine change in `format_text_value` (formatter/format.rs): a text section with `@` plus a date or number token (see the doc) must give `#VALUE!`; A/P and AM/PM give "A"/"AM" literal output. Why handed off: the 4-section case conflicts with the poi evidence that the 4th section prints other tokens as literals (`"x@y;z;w;@@"` is `#VALUE!` in Excel, so the error may come from parsing any section), the session budget ran out, and the baseline for probes/text-lone-surrogates A85-A94 must move with it. Next: probe 2- and 4-section variants with the token in each section, implement, `make eval-baseline`, `make check`, review.
