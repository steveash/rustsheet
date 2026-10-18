---
id: T-1882
title: "A multi-character currency symbol with a locale ([$USD-40C]) prints in date sections"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A multi-character currency symbol with a locale (`[$USD-40C]mmmm`) prints the symbol in a date section and localizes it, as Excel does ("USDmars").

## Context
T-1865 made the one-character `[$€-40C]` work; the lexer's currency path (formatter/lexer.rs) still reads one character, so `[$USD-40C]mmmm` is `#VALUE!`. Failing cell: probes/text-currency-locale-date A14 (expected "USDmars"). Semantics: docs/excel-semantics/number-formats.md "Locale tags localize date names". Probe number sections and multi-character symbols without a locale (`[$USD]0`) first.

## Acceptance criteria
- [ ] probes/text-currency-locale-date A14 passes and any new probes are recorded.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
