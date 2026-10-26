---
id: T-2754
title: "Split the Rust test run across two CI jobs"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The PR CI critical path no longer waits on one runner executing every Rust test: `make test` runs
as two parallel CI jobs that together run exactly the same tests, so the merge lock (decision
0075) is held for a shorter CI run. No gate is dropped or skipped.

## Context
Owner request (2026-10-25). PR CI wall p50 6.7 / p90 7.4 min, bounded by the rust job (p50 5.6,
p90 6.5 min; `make test` 4.8-6.0 min in six PR runs on 2026-10-25). In run 40242056004 the
engine's unit-test binary alone ran 261 s; compiling the tests takes about 1.2 min. Locally the
engine's 4604 unit tests use 585 CPU-s with a long tail (single tests up to 55 s); the modules
test_tailed_array (100), test_fn_groupby (97), fn_regex_backtrack (56) and
test_nested_groupby_cells (21) hold about half. Options weighed (decision 0122): opt-level 1 for
tests (compile +127 s, tests -110 s locally: worse), cargo-nextest partitions (a new tool, a
different runner from `make test`, no doctests), CARGO_INCREMENTAL=0 (dtolnay/rust-toolchain
already sets it), rust-cache keys (PRs already restore main's cache, full match). Peak job
concurrency over the last 60 runs was 24 with no queueing (2 s), so one more job per PR fits.

## Acceptance criteria
- [x] `make test-shard-1` and `make test-shard-2` run disjoint sets of tests whose union is exactly
      what `make test` runs (shown by comparing the three runs' test lists).
- [x] CI's rust job is a two-shard matrix: shard 1 runs fmt-check, clippy and shard 1's tests,
      shard 2 the rest; each shard has its own rust-cache key.
- [x] Before/after job and wall times from real PR runs, in the decision record.
- [x] `make check` green; reviewers approve (security, architecture, quality, performance).

## Out of scope
- The python job (about 4 min, 2.75 of it pytest building the release rustsheet-eval binary).
- The engine build script that re-runs on every cargo call (T-2755).

## Outcome
`make test-shard-1` / `make test-shard-2` split `make test` with libtest filters and the same
substrings as `--skip`; the test lists of the three runs show the shards are disjoint and their
union is the full set (5324 tests, 5310 ok and 14 ignored, statuses identical). CI's rust job is a
two-shard matrix (decision 0122). Shard 1: test_fn_groupby, text_functions::fn_regex,
test_multi_area_arguments, dynamic_evaluation::test_range_reads, functions::text:: (179 tests).
The memory-bound whole-grid tests (test_fn_groupby vs test_tailed_array, test_fn_pivotby) must sit
in different shards: together they slowed shard 1's engine run from 67 s to 138-142 s.
CI (PR #1869; base runs from 2026-10-25): rust job 5.8-7.3 min (`make test` 4.8-6.1) before; the
final split ran 4.3 (shard 1, with clippy) and 4.0 min (shard 2), earlier splits 4.1-4.5 for the
slower shard. PR wall 6.2-8.0 min before; 4.6-5.4 min on this branch, where the eval job (3.5-5.0
min without T-2761) and python (2.8-4.5) now bound it. With T-2761's eval job (2.2 min), the
expected wall is about 0.4 (changes) + 4.3 (rust shard / python) min.
