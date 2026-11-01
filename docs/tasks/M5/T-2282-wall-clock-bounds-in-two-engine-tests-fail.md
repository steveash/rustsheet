---
id: T-2282
title: "Wall-clock bounds in two engine tests fail on a loaded machine"
milestone: M5
lane: quality
status: done
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
- [x] Neither test asserts an absolute wall-clock bound. Each still fails on a quadratic version of
  the code it guards (show that with a deliberate local regression; describe it in the Outcome).
- [x] No test is skipped, ignored or deleted.
- [x] `make check` green; reviewers approve.

## Out of scope
- Speeding up the code under test.

## Outcome
Done (2026-11-01).
- **The regex test** (`brackets_under_case_insensitive_share_one_size_budget`, functions/text/regex_posix.rs) no longer
  exists on main: that file and test were removed before this task was picked up, so only the spill test was left.
- **The spill test** (`test/dynamic_evaluation/test_range_reads.rs`,
  `a_large_spill_is_checked_against_many_rectangles_in_near_linear_time`) now bounds work, not time:
  - `SeenRanges` gains a test-only counter, `rectangles_tested` (`#[cfg(test)]`, a `Cell<u64>`). It counts each
    rectangle a spill commit tests (`SeenRange::first_cell_written`) in `written_by_spill`.
  - The test asserts at most 6 tests per recorded rectangle: three spills commit once per pass, and each commit tests a
    rectangle at most once, over two passes. Today it is 500,000 against a bound of 600,000. The wall-clock assertion
    is gone; the value and count assertions are unchanged.
- **Deliberate regression** (local, not committed):
  - `written_by_spill` was changed to test every rectangle for each spilled row, the cell-by-rectangle scan the test
    guards against.
  - A scaled-down copy of the test (1,000 rectangles, 2,000 spilled rows per pass) fails: "3500000 rectangle tests
    against a bound of 6000". The real code passes it.
  - At full size the same scan makes about 2E+10 tests, so the committed test fails on it with any load.
- No test is skipped, ignored or deleted.
