---
id: T-0929
title: "VALUE: exact percent parsing and d-Mon-yyyy text dates (2 known failures)"
milestone: M9
lane: functions
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`VALUE("23.123%")` and `VALUE("2-Feb-2026")` match Excel.

## Context
Oracle-free evidence in `ironcalc-calc/TEXT/T_VALUE_VALUETOTEXT.xlsx` (4 failing lines):
- `Sheet1!I4` `=VALUE("23.123%")` expected 0.23123, actual 0.23123000000000002: we divide the parsed number by 100; Excel yields the correctly rounded 0.23123 (parse with a decimal exponent shift, i.e. as "23.123e-2").
- `Sheet1!I7` `=VALUE("2-Feb-2026")` expected 46055, actual #VALUE!: day-month-name-year text date form is not recognised.
- NOT in scope: `I5` ("10-12-1999" -> 36504) and `C15` ("2/3/2023" -> 44987) were produced by a day-first (non-en-US) Excel locale; en-US semantics (month-first) must be kept, so those two baseline lines remain known failures. Note this in the PR and in `docs/excel-semantics/`.

## Acceptance criteria
- [ ] `I4` and `I7` lines disappear from `eval/baseline/ironcalc-calc.txt`; no new ones; `I5`/`C15` remain.
- [ ] Unit tests; check whether DATEVALUE shares the text-date parser (one definition; `rg parse_datevalue_text`).
- [ ] `make check` green.

## Out of scope
Locale-dependent date order; T-0920's finite-number parse.
