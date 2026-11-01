# 0242 — The merge train's CI timeout is 15 min because train CI now takes 10 to 11 min

- **Status:** accepted (2026-11-01, T-3488). Amends SPEC-0027 section 4's `CI_TIMEOUT` constant.
- **Context:**
  - SPEC-0027 set `CI_TIMEOUT = 10 min` as "about twice a normal run", when train and PR CI took
    4.9–5.4 min. A wait that passes it is an infrastructure failure: the merger pushes a fresh
    record commit once and, if the second wait passes it too, stops the hold (section 6).
  - On 2026-11-01, 31 completed train CI runs on `merge-train/current` (created after
    2026-10-31 22:50 UTC, the 70-min outlier at 07:54 left out) took p50 10.0, p90 10.3 and at
    most 10.7 min, by the workflow-runs API's `created_at` to `updated_at`. 15 of the 31 took
    10 min or more. CI wall time grew with the test suites; T-3699 to T-3705 shorten it.
  - The waits fail at the timeout. Batch #2719, #2716, #2715, #2720, #2691 on main 8c8cfd39:
    its run created at 12:57:08 UTC passed 10 min, the retry's record commit at 13:07:36
    cancelled it at 13:07:54 (seconds before it would have ended), the retry's run 41309736899
    ended green at 13:18:21, and the merge lock was freed at 13:18:08, before that, with nothing
    merged. Batch #2714, #2729, #2727, #2713, #2725 needed the retry too (11:44 and 11:55 UTC).
  - The spec rejected 15 min because, without a resume, the budget gate cut bisection off and
    livelocked (`sim6.py`, 25–237 runs per merge). Resume is always on now (section 6): a
    cut-off records its search state on the train ref, and the next hold continues from it.
- **Decision:** `CI_TIMEOUT` is 15 min (scripts/merge_train.py). The budget gate is unchanged in
  form: a wait for `n` PRs needs `CI_TIMEOUT + n × MERGE_BUDGET` of the 40-min hold left, so a
  full batch of 5 needs 27.5 min, not 22.5.
- **Consequences:**
  - A train whose CI takes 10–15 min merges on its first run instead of costing a second run and,
    often, the whole hold. A run that takes over 15 min is still an infrastructure failure.
  - A hold that has spent more than 12.5 min (a full batch) before its CI wait is cut off rather
    than starting a wait it might not finish; the next merger run, 30 min later at most, resumes.
    A bisection fits fewer runs per hold; resume carries the search across holds.
  - Revisit (lower it again) once T-3699 to T-3705 bring train CI back under about 7 min; keep the
    timeout at about 1.5× the p90 of the last day's train runs (`make merge-train-stats`).
