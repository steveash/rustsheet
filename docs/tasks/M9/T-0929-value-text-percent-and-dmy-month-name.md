---
id: T-0929
title: "VALUE: exact percent parsing (1 known failure)"
milestone: M9
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`VALUE("23.123%")` matches Excel.

## Context
Oracle-free evidence in `ironcalc-calc/TEXT/T_VALUE_VALUETOTEXT.xlsx` (1 failing line left):
- `Sheet1!I4` `=VALUE("23.123%")` expected 0.23123, actual 0.23123000000000002: we divide the parsed number by 100; Excel yields the correctly rounded 0.23123 (parse with a decimal exponent shift, i.e. as "23.123e-2").
- Done elsewhere (T-0024): `I7` `=VALUE("2-Feb-2026")` now parses (shared date-text grammar), and `I5`/`C15`, whose cached values came from a day-first Excel, are scored against en-US oracle values through `eval/suites/ironcalc-calc/cached-overrides.tsv` (decision 0032).

## Acceptance criteria
- [ ] The `I4` line disappears from `eval/baseline/ironcalc-calc.txt`; no new ones.
- [ ] Unit tests.
- [ ] `make check` green.

## Out of scope
Locale-dependent date order; T-0920's finite-number parse.
