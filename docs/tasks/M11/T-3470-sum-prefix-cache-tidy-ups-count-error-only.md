---
id: T-3470
title: "SUM prefix cache tidy-ups: count error-only entries against the state cap, split prefix_sum"
milestone: M11
lane: perf
status: done
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
- [x] An error-only entry adds at least 1 to `kept`; a test with more such starts than a lowered cap shows the
      cap holds (the walk answers past it, with the same values).
- [x] `prefix_sum` split as above; `test_sum_prefix.rs`'s existing tests pass unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing which SUMs the cache serves.

## Outcome
- **Cap accounting:** `SumPrefixes::extend` merges new states and keeps the `kept` accounting; a new entry adds
  `entry_cost` = at least 1. `limit` (default `MAX_KEPT_STATES`) is a field so tests can lower it.
  - The check before reading reserves room for every state the call may keep, and at least 1 for a new entry.
  - A hit needs no room, including one on an entry that stopped at an error.
- **Refactor:** `prefix_range` (eligibility and key, returning a `PrefixRange` with `cell(offset)`) and the merge
  step are split out of `prefix_sum`. The redundant `caching` binding is gone.
- **Found while testing:** a one-cell range (`SUM($A$1:A1)`) was keyed as a row run, so a column run from the same
  start missed its entry. Values were never affected. One cell now goes to the walk, which reads that one cell.
- **Docs:** the doc comment of the shared runner's `Runs` (`test/pass_cache_runs.rs`, which replaced
  `cached_and_walked` in T-3471) now says what the probe fields hold.
- **Tests:** `entries_without_states_count_against_the_limit` covers 20 error-first starts. They count 20 states at
  the default limit and stay under a limit of 5, with every value still `#DIV/0!`.
  `hits_need_no_room_under_the_limit` and `error_hits_need_no_room_under_the_limit` check that a full limit reads
  the same cells as a roomy one.
- **Mutations caught:** an empty entry counting 0, no hit exemption, and one cell keyed as a run.
