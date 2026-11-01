---
id: T-3707
title: "Replace the wall-clock bounds in the regex and PCRE backtracking tests with step counts"
milestone: M5
lane: quality
status: todo
priority: P3
size: M
tier: sonnet
depends_on: [T-2282]
---

## Goal
No regex or PCRE test asserts an absolute wall-clock bound: the 8 assertions in the three files below bound the matcher's
steps instead, so a catastrophic-backtracking regression still fails and a busy machine does not.

## Context
- Found in T-2282's quality review (#2723). T-2282 replaces the large-spill test's 20 s bound with a count of rectangle
  tests (`SeenRanges::rectangles_tested`, `#[cfg(test)]`); that counter lands with T-2282. T-2282 saw wall-clock bounds
  fail under a load average of about 15 (four `make check`s on four cores); a count is stable where time is not.
- The inventory (`rg -n "elapsed\(\)\s*<|Duration::from" crates/rustsheet-engine/src`, 2026-11-01) is 30 bound
  assertions in 16 test files on main, 29 once T-2282 removes the large-spill one, split by what they guard into T-3706 (evaluation and range walks), T-3707 (regex and
  PCRE backtracking) and T-3708 (function kernels). `test/test_recalc_budget.rs` `worked_example_timing` only prints its
  timing and asserts no bound, so none of the three covers it; `clock.rs` uses durations as inputs, not bounds.
- Reuse a natural counter (the area budget, the recalc budget, a step budget, a loop bound) before adding one; any new
  counter is `#[cfg(test)]` or `pub(crate)`, never public.
- This task's tests:
  - `test/text_functions/fn_regex_backtrack.rs` (5 assertions);
  - `functions/text/pcre/tests.rs` (2);
  - `functions/text/regex_functions.rs` (1).
- These guard exponential backtracking, so the step count (or the matcher's own step budget, if it has one) is the
  natural measure; a regression to the exponential matcher must exceed the bound by orders of magnitude.

## Acceptance criteria
- [ ] `rg -n "elapsed\(\)\s*<|Duration::from" crates/rustsheet-engine/src/test/text_functions/fn_regex_backtrack.rs crates/rustsheet-engine/src/functions/text/pcre/tests.rs crates/rustsheet-engine/src/functions/text/regex_functions.rs` finds no bound assertion; each guard asserts a work count
      or a size ratio instead.
- [ ] Each still fails on a deliberate local regression of the code it guards (described in the Outcome).
- [ ] No test is skipped, ignored or deleted.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Speeding up the code under test. Benchmarks (`benches/`) keep their timings.
- The other two tasks' files.
