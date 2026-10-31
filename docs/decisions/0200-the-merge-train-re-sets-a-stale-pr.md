# 0200 — The merge train re-sets a stale PR's base so GitHub recomputes its test merge

- **Status:** accepted (2026-10-31, T-3455)
- **Context:**
  - The first live train (2026-10-31 11:51 UTC, `merge-train/current` run 41198551221) tested
    #2533, #2524 and #2546 green, then stopped before squashing. The stop was step 7.2:
    "GitHub's test merge of #2533 was not recomputed on 4ff408243c5e within 2 min".
  - GitHub recomputes a PR's test merge (`merge_commit_sha`) when the PR changes, not when its
    base branch moves. Every open PR's test merge sat on whatever `main` was at its last push:
    #2533 on a3df1f16, #2104 and #2373 on b1bdc1c2, #2524 on 1248554a. Polling did not help;
    #2533 stayed stale for over 20 minutes.
  - Re-setting a PR's base to the branch it already has (`PATCH pulls/{n}` with `base: main`)
    made GitHub recompute #2104's test merge on the current tip within about 3 minutes.
  - The tests' fake GitHub recomputed the test merge on every read, which is why no test caught
    this. Shadow mode never squashes, so step 7.2 had never run live.
- **Decision:**
  - When step 7.2 finds a test merge on an older `main` while `main` has not moved, the stage
    re-sets the PR's base to `main` once (`Client.set_base`) and keeps polling.
    `PRE_SQUASH_TIMEOUT` grows from 2 to 5 minutes to cover the recompute.
  - Step mode gains an eighth action, `set_base` (fields `pr`, `base`; `base` must be `main`). Its
    read-back checks that the PR's `base.sha` equals the tip of `main`. The merger prompt in
    docs/process/routines.md lists it: the session performs it with `update_pull_request`
    (`base: main`).
  - The fake GitHub in scripts/test_merge_train.py keeps a stale test merge until the base is
    re-set (`test_bases`). A new test drives a stale #2 through the refresh.
- **Consequences:**
  - Re-setting a base changes nothing on the PR besides the recompute. It is the same base, and
    GitHub records no event that the readiness checks read.
  - Tree safety is unchanged: the recomputed test merge must still sit on `main` and have the
    train's tree, or the stage stops (with a P0 for a different tree).
  - A train now costs up to one extra API write and a few minutes per member whose test merge
    is stale.
  - The owner updates the merger routine's prompt to the routines.md text (owner step 3 of
    SPEC-0027). Until then, a merger run refuses `set_base` (`REFUSED=1`), and the stage stops at
    that member as it did before this change. Sessions running the stall fallback perform it.
