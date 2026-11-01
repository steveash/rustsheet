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
  - The spec rejected a 15-min `CI_TIMEOUT` because the budget gate reserves `CI_TIMEOUT` per
    wait: at 15 min it cut the section 6 bisection example off at its fourth run
    (scripts/test_merge_train_red.py checks those examples, and fails with the gate at 15 min).
- **Decision:** the gate's reservation and the wait's limit become two constants
  (scripts/merge_train.py). `CI_RESERVE` = 10 min is what the gate reserves: a wait for `n` PRs
  starts only with `CI_RESERVE + n × MERGE_BUDGET` of the hold left, as before. `CI_TIMEOUT` =
  15 min is the longest a wait may take, but a wait runs past `CI_RESERVE` only into hold budget
  that its `n` merges do not need: its limit is `max(CI_RESERVE, min(CI_TIMEOUT, left − n ×
  MERGE_BUDGET))`, read when the wait starts.
- **Consequences:**
  - In a fresh hold (40 min) a train whose CI takes 10–15 min merges on its first run instead of
    costing a second run and, often, the whole hold. A run past its limit is still an
    infrastructure failure.
  - The gate and the bisection budgets of section 6 are unchanged, so a hold fits as many CI
    waits as before; a late wait in a long bisection may still stop at 10 min and resume in the
    next hold.
  - Revisit (go back to one 10-min constant) once T-3699 to T-3705 bring train CI back under
    about 7 min. T-3736 stops the timeout retry from cancelling a run still in progress.
