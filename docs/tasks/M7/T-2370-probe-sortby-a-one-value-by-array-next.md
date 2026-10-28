---
id: T-2370
title: "Probe SORTBY: a one-value by_array next to a row by_array on a one-row array, and the sort_order snap tolerance"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence settles the two SORTBY cases T-2310 left unprobed, and the code and docs/excel-semantics/arrays.md follow it.

## Context
T-2310 (probes/t2310-sortby-single-by-array) found that a by_array sorts rows when it is a column as tall as the array and
columns when it is a row as wide as it, with every key agreeing, and that a sort_order reads as `near_integer_count`. Two parts
are guesses (arrays.md "SORTBY with a single-value by_array or a fractional sort_order"): (1) on a one-row array, a one-value
by_array next to a row by_array (`SORTBY(A20:C20,5,1,A20:C20,1)`, and the reverse order) is `#VALUE!` in ours, since we count
the one value as sorting rows only; (2) the snap tolerance of a fractional sort_order is LEFT's 2^-22 + 2^-33, bounded only by
-1E-300 and 0.9999. Probe both: the mixed calls, and sort_orders 1-2^-22, 1-2^-23, -1+2^-22, -1+2^-23, 1-2^-21. Code:
`by_array_axes` and `sortby_of` in functions/spill_functions.rs.

## Acceptance criteria
- [ ] A probe case answers both questions (excel-graph) and passes; arrays.md states the measured rule, without UNVERIFIED for these two.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SORT's own sort_order fractions.
