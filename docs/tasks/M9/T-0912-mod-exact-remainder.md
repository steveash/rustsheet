---
id: T-0912
title: "MOD uses the exact remainder (fmod) like Excel"
milestone: M9
lane: functions
status: done
priority: P3
size: S
depends_on: []
---

## Goal
`MOD(n, d)` differs from Excel for tiny divisors / near-integer ratios because it used
`n - d*floor(n/d)`. Excel's cached values match the exact remainder.

## Acceptance criteria
- [x] MOD_QUOTIENT.xlsx C20/C21/C25/C26 pass (Excel-cached evidence).
- [x] Unit test; semantics entry in `docs/excel-semantics/numbers.md`.

## Outcome
Switched to `fmod` plus sign shift; all four cells match bit-for-bit.
