# 0239 — CI runs on main are not cancelled by the next merge

- **Status:** accepted (2026-11-01, owner request after a CI investigation, T-3704). It amends
  0054 (the eval report runs on pushes to main) and the workflow's concurrency rule.
- **Context:**
  - `ci.yml` cancels an older run of the same ref when a newer one starts (`concurrency:
    ci-${{ github.ref }}`, `cancel-in-progress: true`).
  - On main, merges land faster than CI finishes. Of the 16 runs on main in the 100 CI runs of
    2026-11-01 05:53-09:16Z, 12 were cancelled after 0.2-5.6 min (about 25 runner-minutes), and
    only 4 finished.
  - So most merges never get decision 0054's per-merge eval report, and most landed shas are never
    tested as they are.
    - In `train` mode the train's CI tests a merged tree, but the merger squash-merges each PR as
      a new commit on main.
    - In `lock` mode, and for anything pushed outside the train, the run on main is the only run
      of that sha.
- **Decision:** `cancel-in-progress` is `${{ github.ref != 'refs/heads/main' }}`.
  - A PR's or a train's older run is still cancelled by a newer push to that ref.
  - A run on main finishes once it starts.
  - Main keeps its full run, gates included: trimming main to the report alone would drop the only
    test of the landed sha when the train is off, so T-3704's optional trim is not taken.
- **Consequences:**
  - GitHub keeps at most one pending run per concurrency group. A burst of merges therefore
    queues behind the running one, and a newer pending run replaces an older pending run. Every
    run that starts finishes, but a merge in the middle of a burst can still be skipped, and the
    sha after it covers it.
  - Main's runs use more runner time than the cancelled ones did, about one 6-7 min run per
    merge that starts. Concurrency stayed far below the runner limit on 2026-11-01 (0122).
  - A red run on main now reliably reports (it is no longer cancelled before failing). The
    steward's "CI on main is green" health line reads a finished run.
