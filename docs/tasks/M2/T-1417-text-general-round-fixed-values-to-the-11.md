---
id: T-1417
title: "TEXT General: round fixed values to the 11-character limit"
milestone: M2
lane: functions-text
status: todo
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
`TEXT(x,"General")` rounds fixed values to Excel's 11-character limit (sign excluded, decimal point counted) for every |x| in [1E-09, 1E+11), not only below 1.

## Context
`format_general` (crates/rustsheet-engine/src/formatter/format.rs) keeps 9 decimals from 1 on. Oracle
evidence (T-1225, docs/excel-semantics/number-formats.md): `123456.123456789` → "123456.1235",
`22.712470704` → "22.7124707", `9999999999.5` → "10000000000", `99999999999.6` → "1E+11".
Failing cells: probes/text-general-ties-wide-fixed Calc!A4-A10, A15-A17. About 30 engine unit tests
(e.g. test_convert, test_database) pin the 9-decimal form; update them to the oracle forms in the
same PR. Mind the digit-count comparison against E notation after rounding.

## Acceptance criteria
- [ ] probes/text-general-ties-wide-fixed has no known failures in eval/baseline/probes.txt (rerun `make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Number-to-text coercion (`&""`, 15 digits; numbers.md).
- Split the task if the unit-test updates grow beyond the `format_general` change.
