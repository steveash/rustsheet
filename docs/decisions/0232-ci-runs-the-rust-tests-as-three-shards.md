# 0232 — CI runs the Rust tests as three shards

- **Status:** accepted (2026-11-01, owner request to shorten PR CI, T-3699). It amends 0122: the
  same tests run, in three CI shards instead of two.
- **Context:**
  - Over the 100 CI runs of 2026-11-01 (05:53-09:16Z), heavy PR runs took p50 10.2 / p90
    10.8 min. `rust (test 2/2)` was the critical path: its `make test-shard-2` step took p50 544 s,
    p90 577 s (10 sampled runs, e.g. 41283701203 at 577 s). `make test-shard-1` took p50 172 s,
    plus 39 s of clippy. Decision 0122 asks for a rebalance when one shard runs more than 1.5 min
    longer than the other; the gap was about 5.5 min.
  - Cause: `test::test_tailed_array` grew from about 100 test-s (0122) to 988 of the 2,184
    test-s that `make test` spends locally (4 threads; `--report-time` on 2026-11-01). Each of
    its tests evaluates every formula twice, once tailed and once over the full 1,048,576-row
    arrays as the reference, in a debug build. It all ran in shard 2, the default shard. Single
    tests take up to 149 s (`index_picks_from_a_tailed_argument`).
  - The other jobs take python 5.1 min, eval 4.2 and hygiene 2.5, so two balanced shards (about
    6.5 min each) would still bound the run, and three shards of about 4.5-5.5 min would not, by
    much.
- **Decision:**
  - `make test-shard-1` runs the tests matching a `TEST_SHARD_1` substring. `make test-shard-2`
    runs those matching `TEST_SHARD_2` and not `TEST_SHARD_1`. `make test-shard-3` runs every
    other test, using `--skip` on both lists. Together they run exactly the tests of `make test`,
    each once, and a new test lands in shard 3. On 2026-11-01 the three test lists held 651, 44
    and 5,359 tests with no overlap, and their union equals `make test --list`: 6,054 tests.
  - The split comes from the measured times, in local test-seconds with 160 added to shard 1 for
    fmt-check and clippy: 803, 748 and 793.
    - Shard 1 keeps 0122's list and adds `test::lookup_and_reference`, `test::test_collation`,
      `test::test_formula_work` and `test::statistical`.
    - Shard 2 takes `test::test_fn_pivotby`, `test::test_nested_groupby_cells` and the
      `test_tailed_array` tests named `keyed*`, `index*`, `lookup*` and `small_and_large*`
      (493 test-s).
    - Shard 3 has the rest of `test_tailed_array` (495 test-s) and everything else.
  - 0122's memory-bound rule stands: `test_fn_groupby` stays in shard 1, apart from
    `test_fn_pivotby` and every `test_tailed_array` test. `test_tailed_array` is split between
    shards 2 and 3, so no runner holds all of it.
  - CI's rust matrix is `shard: [1, 2, 3]`, one rust-cache key per shard. The check names are
    `rust (fmt, clippy, test 1/3)`, `rust (test 2/3)` and `rust (test 3/3)`. The required check
    is still the summary `rust (fmt, clippy, test)` of 0122, so no required-check name changes.
- **Consequences:**
  - Every gate still runs on every PR, and `make test` and `make check` are unchanged.
  - Each heavy PR uses one more runner, and each shard compiles the tests (about 1-1.5 min in CI).
    Peak concurrency stays far below the runner limit: 0122 measured 24 jobs with no queueing.
  - The steward's rebalance rule from 0122 now compares the slowest and fastest of the three
    shards. When it rebalances, it adds substrings to `TEST_SHARD_1` or `TEST_SHARD_2` and keeps
    the memory-bound rule above. A substring that stops matching moves its tests to shard 3 and
    drops none.
  - The tests themselves are not made faster here. A debug build's full-array reference
    evaluations dominate `test_tailed_array`; making them cheaper is T-3709.
