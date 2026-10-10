---
id: T-1225
title: "Probe TEXT General fixed-versus-E ties and wide fixed values"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence settles the cases where T-0938's `TEXT(x,"General")` rule is the planner's
reading rather than probed fact: ties in significant digits between the fixed and E forms, and
fixed values wider than 11 characters in the integer-plus-fraction range.

## Context
T-0938 implemented `format_general` (crates/rustsheet-engine/src/formatter/format.rs): fixed
notation with 9 decimals (trimmed) when `1E-09 <= |x| < 1E+11` and it shows at least as many
significant digits as E notation; otherwise E notation. Ties go to
fixed. Every TEXT/General cell in the suites passes, but none of them tests these cases:
- Multi-digit ties (single-digit ties already go to fixed, poi/GeneralFormatTests A27):
  `0.000123456789` (fixed "0.000123457" vs "1.23457E-04", 6 digits each; the rule gives
  fixed), `1.2345E-05` (5 vs 5; fixed "0.000012345"), `1.5E-09` (fixed "0.000000002", 1 digit,
  against "1.5E-09"; the rule gives E).
- Wide fixed values: the engine keeps 9 decimals from 1 on, so `123456.123456789` prints
  "123456.123456789" (16 characters). If Excel's 11-character limit applies here too it would be
  "123456.1235". Also probe `-123456.123456789`, `22.712470704`, `9999999999.5`,
  `99999999999.6` and `12345678901.5`. A change here moves about 30 engine unit tests that pin
  the 9-decimal form (e.g. test_convert, test_database), so update them in the same PR.
Write a probe case (excel-probe skill), get the oracle values, and record them in
`docs/excel-semantics/number-formats.md` ("`TEXT(x,"General")` uses an 11-character
fixed-notation limit"). If Excel disagrees, file an engine task with the failing cells.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes with the cells above, oracle-filled (excel-graph).
- [ ] number-formats.md records the results; any mismatch has its own engine task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Number-to-text coercion (`&""`, 15 digits; numbers.md).

## Outcome
Probe `text-general-ties-wide` oracle-filled (excel-graph). Multi-digit ties and `1.5E-09` match the engine; wide fixed values do not (Excel limits to 11 characters). Recorded in number-formats.md; engine mismatch filed as T-1298. Eval delta: probe cells added, new known failures B4:B9 (baseline).
