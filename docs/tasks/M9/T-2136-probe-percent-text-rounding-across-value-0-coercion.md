---
id: T-2136
title: "Probe percent text rounding across VALUE, 0+ coercion and typed input"
milestone: M9
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence that percent number text is rounded once (decimal shift) on every text-to-number path, not only VALUE.

## Context
T-0929 made `parse_percent_or_currency` (formatter/parse_formatted.rs, `percent_value`) read `N%` as N's decimal
text shifted two places. Only `VALUE("23.123%")` is evidenced (ironcalc-calc T_VALUE_VALUETOTEXT.xlsx `I4`);
`0+"23.123%"`, typed cell input `23.123%` and e-notation (`23e-12%`) share the parser but are unprobed
(docs/excel-semantics/coercion.md "Percent text is rounded once").

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with `VALUE`, `0+` and `ISNUMBER`/typed input for `23.123%`, `-23.123%`,
  `0.07%`, `1.005%`, `12345.6789012345678%`, `23e-12%`, `1.5E-300%`, oracle-filled and passing (or failures filed).
- [ ] coercion.md entry updated with the verified paths.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The formula `%` operator (`=23.123%`).
