---
id: T-3470
title: "SUM prefix cache tidy-ups: count error-only entries against the state cap, split prefix_sum"
milestone: M11
lane: perf
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Every entry of the SUM prefix cache (`functions/math_and_trigonometry/sum_prefix.rs`, T-3456) counts against
`MAX_KEPT_STATES`, an entry that stopped at an error in its first cell included, and `prefix_sum` reads as
eligibility, extension and merge steps.

## Context
Review of T-3456 (PR 2593), MINOR findings deferred:
- security: an entry whose first cell is an error has zero states and adds 0 to `kept`; such entries are
  bounded only by the number of distinct starts (one per formula). Count each entry as at least 1.
- quality: `prefix_sum` is about 120 lines; split out the eligibility/key step and the cache merge (a
  `SumPrefixes::extend(key, start, states, error)` keeping the `kept` accounting with the data); `caching`
  is bound `true` in every arm of its match; the doc comment of `cached_and_walked` in
  `test/mathematical/test_sum_prefix.rs` should say what the returned tuple holds.

## Acceptance criteria
- [ ] An error-only entry adds at least 1 to `kept`; a test with more such starts than a lowered cap shows the
      cap holds (the walk answers past it, with the same values).
- [ ] `prefix_sum` split as above; `test_sum_prefix.rs` unchanged and passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing which SUMs the cache serves.
