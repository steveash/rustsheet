---
id: T-2282
title: "Wall-clock bounds in two engine tests fail on a loaded machine"
milestone: M5
lane: quality
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`make check` passes on a loaded machine: the two engine tests that assert on elapsed time keep
guarding against a quadratic (or unbounded) regression without failing when the box is busy.

## Context
- On 2026-10-21 the steward ran four `make check`s at once on a 4-core sandbox (load average about
  15). Three runs failed on the same two tests, and each passed alone in about 6 s:
  - `functions/text/regex_posix.rs` `brackets_under_case_insensitive_share_one_size_budget`
  - `test/dynamic_evaluation/test_range_reads.rs`
    `a_large_spill_is_checked_against_many_rectangles_in_near_linear_time` (28 s against its bound)
- CI runners are less loaded, but a busy runner would make these flaky. A count of steps (work
  done, comparisons, cells charged) is a stable bound where wall-clock time is not; a ratio
  between a small and a large input is another option.

## Acceptance criteria
- [ ] Neither test asserts an absolute wall-clock bound. Each still fails on a quadratic version of
  the code it guards (show that with a deliberate local regression; describe it in the Outcome).
- [ ] No test is skipped, ignored or deleted.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Speeding up the code under test.
