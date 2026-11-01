# 0233 — make py-check runs pytest on every core

- **Status:** accepted (2026-11-01, owner request to shorten PR CI, T-3700). It changes how the
  tools/ tests run, not which tests run.
- **Context:**
  - The python CI job took p50 306 s on 2026-11-01 (10 sampled heavy PR runs); its `make
    py-check` step was p50 242 s. With the Rust tests in three shards (decision 0232), python
    (5.1 min) is within a minute of the slowest shard.
  - Locally `py-check` is almost all pytest: ruff takes 0.3 s, mypy 6 s and pytest 117 s, all
    1,248 tests in one process. The slowest tests are fitlab solver scores (up to 10 s each) and
    the generator's coverage tests (about 4 s each).
  - Every agent runs the same target in `make check` before each PR.
- **Decision:**
  - `pytest-xdist` (>= 3.6; 3.8.0 and its dependency execnet 2.1.2 in `tools/uv.lock`) is a dev
    dependency of tools/. Both are pytest-dev projects under the MIT license.
  - `make py-check` runs `pytest -q -n auto`: one worker per CPU, with the same tests and the same
    assertions.
  - Plain `uv run pytest` stays serial (no `addopts`), so running one test stays cheap.
  - `accuracy.eval_math.write_suite` creates its process pool with the `spawn` context. Under
    xdist the default `fork` ran in a multi-threaded worker and raised Python's "use of fork()
    may lead to deadlocks" warning. `spawn` is deadlock-safe and exists on every platform
    (the oracle runs on Windows).
- **Measured (local, 4 cores, 2026-11-01):** 1,248 tests passed either way, with no warnings.
  - Before: 117 s serial on an idle machine. A second serial run, slowed by other jobs on the same
    machine, took 440 s.
  - After: 56-64 s with `-n auto`.
  - CI times are in T-3700's Outcome.
- **Consequences:**
  - A test that shares mutable state across tests (a fixed temp path, a module-level cache it
    mutates, a fork from a threaded parent) can now fail or race under `make check`. Such tests
    must use `tmp_path` and monkeypatch, as the suite already does.
  - The release `rustsheet-eval` binary that `rustsheet_tools.snapshot` builds on demand is safe
    when several workers ask for it at once: cargo's own build lock serializes them.
