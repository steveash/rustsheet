---
id: T-1817
title: "Probe REGEXREPLACE results past 32,767 characters"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's result for a REGEXREPLACE result past the 32,767-character text limit is probed and ours matches it.

## Context
T-1272 (performance and security reviews) already sizes the result before building it: `Replacement::result_len` in
functions/text/regex_functions.rs, and a result over 32,767 UTF-16 units is `#VALUE!`, as SUBSTITUTE's is
(text.md "Lifted TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* calls are capped", UNVERIFIED). Use the excel-probe skill:
`=LEN(REGEXREPLACE(REPT("a",32767),"","bbbbbbbbbb"))`, `=LEN(REGEXREPLACE(REPT("a",32767),"a","bb"))`, a result of
exactly 32,767 and 32,768 units, and one with an emoji straddling the limit. If Excel truncates or gives another error,
change `too_long` in regex_functions.rs to match.

## Acceptance criteria
- [ ] A probe case with oracle values covers the cells above; text.md marks the rule VERIFIED or records Excel's.
- [ ] Ours matches; `make check` green; reviewers approve.

## Out of scope
- The bound itself (T-1272).
