# 0122 — CI runs the Rust tests as two parallel shards

- **Status:** accepted (2026-10-25, owner request to shorten PR CI, T-2754). It changes where the
  Rust tests run, not which tests run.
- **Context:** PR CI wall time is how long the merge lock (decision 0075) is held. Over the 48 h
  before 2026-10-25 PR runs took p50 6.7 / p90 7.4 min, bounded by the rust job (p50 5.6, p90 6.5
  min). Its `make test` step took 4.8-6.0 min in six PR runs that day; in run 40242056004 the
  engine's unit-test binary alone ran 261 s, after about 1.2 min of compiling. The tests are CPU
  bound (4604 engine unit tests, 585 CPU-s locally on 4 threads, single tests up to 55 s), so only
  more cores help. Options weighed with numbers:
  - opt-level 1 for tests: locally compile +127 s and tests -110 s, a net loss;
  - cargo-nextest partitions: a new CI tool and a different test runner from `make test` (one
    process per test, no doctests), so CI would no longer run what agents run;
  - `CARGO_INCREMENTAL=0`: dtolnay/rust-toolchain already sets it;
  - rust-cache keys: PRs already restore main's cache with a full match;
  - splitting `make test` with libtest's own filters: no new tool, same runner.
  Peak concurrency over the last 60 runs was 24 jobs with no queueing (2 s), so one more job per
  PR fits.
- **Decision:** `make test-shard-1` runs the tests whose names contain one of the substrings in
  `TEST_SHARD_1` (the slowest engine modules, about half the test CPU time); `make test-shard-2`
  runs every other test with `--skip` on the same substrings, which libtest matches the same way.
  Together they run exactly the tests of `make test`, each once; a new test lands in shard 2
  unless it matches. CI's rust job is a matrix of the two shards; shard 1 also runs fmt-check
  and clippy, and each shard has its own rust-cache key. A summary job keeps the old check name
  `rust (fmt, clippy, test)`: it passes only when both shards passed, or when a docs-only PR
  (decision 0049) skipped them, and fails on any other result (a failed, cancelled or missing
  shard). It is the one rust check to require, so the required-check list in decision 0053 stays
  as it is and a later change to the shard count renames nothing that is required. `make test` and `make check` are
  unchanged.
- **Measured (PR #1869, four runs):** the slower shard took 4.1-4.5 min against 5.8-7.3 min for
  the old rust job; the final split ran 4.3 and 4.0 min. Memory-bound whole-grid tests
  (test_fn_groupby's 1M-row visit-cap tests, test_tailed_array's whole-column ones) slow each
  other down when one shard holds both (shard 1's engine run went from 67 s to 138-142 s), so
  `TEST_SHARD_1` keeps them apart. PR wall went from 6.2-8.0 to 4.6-5.4 min, then bounded by the
  eval job (fixed by decision 0125) and the python job.
- **Consequences:** every gate still runs on every PR: fmt-check, clippy and the tests of shard
  1 in `rust (fmt, clippy, test 1/2)`, the remaining tests in `rust (test 2/2)`. Each PR uses one
  more runner, and both shards compile the tests. The steward rebalances `TEST_SHARD_1` when the slower
  shard runs more than 1.5 min longer than the other in a day's PR runs (the job durations of
  `rust (fmt, clippy, test 1/2)` and `rust (test 2/2)`); a substring that stops matching only moves tests to shard 2. With
  the rust job shorter, the python job (about 4 min, mostly pytest building the release
  rustsheet-eval binary) bounds PR CI next.
