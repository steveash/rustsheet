# 0204 — The merge train builds from up to 20 ready PRs so ejections do not shrink a batch

- **Status:** accepted (2026-10-31, T-3476; the owner asked for the fix)
- **Context:** step 2 of SPEC-0027 section 4 handed the build only the `BATCH_MAX` (5) oldest
  ready PRs. The build ejects those that conflict with `main` or collide on an id, and defers
  those that conflict within the batch. It never pulled in the next ready PR. On 2026-10-31,
  with 21 PRs ready, a train ejected four of its five candidates and merged a batch of two
  (#2548, #2568).
- **Decision:**
  - Step 2 hands the build up to `CANDIDATES_MAX` = 4 × `BATCH_MAX` (20) ready PRs, oldest marker
    first (`candidates()` in scripts/merge_train.py).
  - The build already stops once it has `BATCH_MAX` members, so a batch still holds at most 5
    PRs, and the lock records only the members.
  - The dry run lists every ready PR it did not place as waiting for a later batch.
- **Consequences:**
  - Ejected and deferred candidates make room for later ones. Each extra candidate costs one
    local merge, plus an id check when it merges cleanly, and ejections happen one train
    earlier.
  - Ordering is unchanged: the oldest marker first, and a later PR never takes the place of an
    earlier one that merges cleanly.
