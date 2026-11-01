---
id: T-3699
title: "Rebalance the Rust CI tests into three shards"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The slower Rust test shard no longer bounds PR CI at about 9.5 minutes: `make test` runs as three
CI shards of similar length that together run exactly the tests `make test` runs, each once. No
gate is dropped or skipped.

## Context
Owner request (2026-11-01, CI-speed investigation). Over the 100 CI runs of 2026-11-01 05:53-09:16Z,
heavy PR runs took p50 10.2 / p90 10.8 min. `rust (test 2/2)` is the critical path: its `make
test-shard-2` step took p50 544 s, p90 577 s (10 sampled runs, e.g. 41283701203: 577 s), while
`make test-shard-1` took p50 172 s plus 39 s of clippy. Decision 0122 asks for a rebalance when the
slower shard runs more than 1.5 min longer than the other; the gap is now about 5.5 min, because new
tests land in shard 2 by default. Shard 2 also runs in two modes (304-322 s in 4 runs, 542-595 s in
6), independent of branch content. Constraint from 0122: the memory-bound whole-grid tests
(test_fn_groupby, test_tailed_array, test_fn_pivotby) must sit in different shards. The other jobs
take python 5.1 min, eval 4.2, hygiene 2.5, so three shards around 4-4.5 min bring the Rust jobs
below python (T-3700 shortens python).

## Acceptance criteria
- [ ] `make test-shard-1`, `make test-shard-2` and `make test-shard-3` run disjoint test sets whose
      union is exactly what `make test` runs (shown by comparing the test lists of the four runs).
- [ ] The split is chosen from measured per-test times (recorded in the decision record), with the
      memory-bound whole-grid tests in different shards; a new test still lands in a default shard.
- [ ] CI's rust matrix has three shards with their own rust-cache keys; `rust (fmt, clippy, test)`
      still passes only when every shard passed or a docs-only PR skipped them.
- [ ] A decision record amends 0122 (protected paths: Makefile, .github/), with before/after
      shard times from real PR runs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- cargo-nextest or any other new test runner (decision 0122 weighed it).
- The python job (T-3700) and path-scoped job selection (T-3701).
