---
id: T-2691
title: "BASE: a written-empty min_length is omitted (BASE(0,2,) is '0')"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`BASE(0,2,)` gives "0" as in Excel: a written-empty `min_length` is read as omitted (default 1), not 0.

## Context
Oracle cells probes/base-zero-empty-min-length A1 and A5 fail (T-2416); a blank-cell `min_length` (Z99) stays 0. See docs/excel-semantics/numbers.md. Code: `fn_base` in mathematical.rs, `kernel_number_or`.

## Acceptance criteria
- [x] probes/base-zero-empty-min-length passes in full; baseline updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- A blank-cell `min_length` (stays 0).

## Outcome
`fn_base` reads a written-empty `min_length` (`CalcResult::EmptyArg`) as omitted, so `BASE(0,2,)` is "0"; a blank-cell `min_length` stays 0. probes/base-zero-empty-min-length A1 and A5 now pass; baseline shrank by 2 known failures. Quirk recorded in docs/excel-semantics/numbers.md.
