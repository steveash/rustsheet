---
id: T-1864
title: "A currency tag with a locale localizes date names"
milestone: M2
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A currency tag with a locale (`[$€-40C]`) prints its symbol and also localizes the section's date names and counts as the section's locale tag, as Excel does.

## Context
`TEXT(45000,"[$€-40C]mmmm")` is "€mars" and `TEXT(45000,"[$-40C]mmmm [$€-407]mmmm")` is `#VALUE!` (a second locale tag). The lexer's currency path (formatter/lexer.rs, `Token::Currency`) drops the locale part and date sections ignore the currency. Failing cells: probes/text-locale-date-names A116, probes/text-locale-name-tables A574. Reuse `locale_tag::resolve` (T-1716). Probe where the symbol goes in a date section (`mmmm[$€-40C]`, `[$€-40C]d mmmm`) first. See docs/excel-semantics/number-formats.md "Locale tags localize date names".

## Acceptance criteria
- [ ] The two cells above pass and the new probes are recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
