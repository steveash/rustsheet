---
id: T-1883
title: "A multi-character currency symbol with a locale ([$USD-40C]) prints in date sections"
milestone: M10
lane: functions-text
status: in-progress
priority: P3
size: S
tier: sonnet
campaign: semantics
depends_on: []
---

## Goal
A multi-character currency symbol with a locale (`[$USD-40C]mmmm`) prints the symbol in a date section and localizes it, as Excel does ("USDmars").

## Context
T-1865 made the one-character `[$€-40C]` work; the lexer's currency path (formatter/lexer.rs) still reads one character, so `[$USD-40C]mmmm` is `#VALUE!`. Failing cell: probes/text-currency-locale-date A14 (expected "USDmars"). Semantics: docs/excel-semantics/number-formats.md "Locale tags localize date names". Probe number sections and multi-character symbols without a locale (`[$USD]0`) first.

Update (T-2157, 2026-10-21): the lexer reads a multi-character `[$...]` symbol as text, so probes/text-currency-locale-date A14 ("USDmars") passes and left the baseline; the probes below remain.

## Acceptance criteria
- [x] probes/text-currency-locale-date A14 passes and any new probes are recorded.
- [ ] Probe also: `[$€-]mmmm [$-40C]mmmm` and `[$€-zzz]mmmm [$-40C]mmmm` (an empty or unresolvable locale on a currency tag; T-1865 makes them count as the section's one tag, unprobed) and a length-limit series built from `[$€-40C]` (T-1865 charges it the locale cost on top of the symbol, unprobed).
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: a wrong TEXT or displayed-text result is visible to every user of the format, so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.
