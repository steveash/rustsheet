---
id: T-2570
title: "DB with a life past i64 overflows the last-period check (nightly fuzz crash)"
milestone: M9
lane: functions-financial
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
`DB` never panics on an untrusted `life`: `=DB(6912,0,1E+300,2,6)` evaluates instead of aborting
the fuzz target (CLAUDE.md directive 5).

## Context
The nightly fuzz run on f132c0049 (run 40059742446, artifact
`fuzz-crash-formula_eval-40059742446`) aborted `formula_eval` on `DB(6912,0,1E+300,2,6)`.
`db_depreciation` (`functions/financial/financial_depreciation.rs`) casts `life.trunc()` to `i64`,
which saturates to `i64::MAX` for 1E+300, and then compares `int_period == int_life + 1`: the add
overflows, a panic under the fuzz build's overflow checks (a release build wraps, so its value is
unchanged). T-2351 was an earlier, different fuzz crash.

## Acceptance criteria
- [x] The last-period check cannot overflow for any finite or infinite `life`; release values are
      unchanged (no eval delta).
- [x] A unit test evaluates the fuzz input and a huge `life` with salvage (rate 0) under the test build's
      overflow checks.
- [x] `make check` green; reviewers approve.

## Out of scope
- Excel's value for a life past 2^53 (unprobed; the result is unchanged by this task).

## Outcome
`db_depreciation` compares `int_period - 1 == int_life` (`int_period` is in 2..=100,000 there, so
no overflow) instead of `int_period == int_life + 1`. Test `fn_db_huge_life_does_not_overflow`
panicked with "attempt to add with overflow" before the fix and passes after; release values are
unchanged, so no eval delta.
