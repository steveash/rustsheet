---
id: T-1467
title: "Regression test for the fuzz FIXED huge-decimals OOM (nightly fuzz run 38344232074)"
milestone: M2
lane: functions-text
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
The input from nightly fuzz run 38344232074 (`formula_eval`) is a unit test that passes. It must not allocate a string with millions of characters again.

## Context
Nightly fuzz run 38344232074 (2026-10-13, on 84d1ab59) failed on `formula_eval` with a deadly
signal (peak RSS 932 MB). The input was
`ENCODEURL(CONCATENATE(FIXED(Data!E9,123456789.123,TRUE),(Data!D3&Data!$C2)))`: FIXED with about
1.2e8 decimals built a string of over 100 MB before CONCATENATE and ENCODEURL copied it again. T-1281
(#672, a852c83) then capped FIXED/DOLLAR decimals (128 or more is `#VALUE!`,
docs/excel-semantics/text.md), and the steward run confirmed the formula now returns `#VALUE!` on
a852c83. Decision 0035 asks for each fuzz reproducer to be kept as a unit test. T-1281 added probe
cells for decimals 128, but no test with this input, so it is the finding that needs the test.

## Acceptance criteria
- [ ] A unit test (next to the FIXED/DOLLAR tests or in the facade's fuzz-regression tests, wherever earlier reproducers such as T-1419's live) evaluates the exact input above on a sheet named `Data` and asserts `#VALUE!` with no panic.
- [ ] The same test covers `DOLLAR(0,123456789)`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to FIXED/DOLLAR semantics.
