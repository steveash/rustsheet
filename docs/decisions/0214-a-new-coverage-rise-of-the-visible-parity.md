# 0214 — A new-coverage rise of the visible-parity lock keeps main's slack

- **Status:** superseded by decision 0219 (the lock is per case, so there is no shared slack).
  Was: accepted (2026-10-31, T-3539), amending decision 0093.
- **Context:**
  - Decision 0093 lets the visibly-off count sit below `eval/baseline/visible-parity.json`. The
    gap is slack, which the steward takes back once a day with `--lower-visible-lock`.
  - A probe PR whose new cases add visibly-off cells still has to write the lock. Until now
    `update-baseline` wrote the run's exact count, so the PR's lock was main's *count* plus the new
    cells, not main's *lock* plus the new cells, and main's slack disappeared on the branch.
  - Main keeps gaining cells inside its own slack: probe cases that fit it and already-failing
    cells that turn visibly off. Each of those breaks the PR's lock once the train merges the PR
    onto main, even though the PR itself changed nothing about them. On 2026-10-31, #2581 (T-3298,
    130 new visibly-off cells) was ejected for this (`ci-red`, visible parity). After that, every
    merge of `main` needed a fresh re-record, and the lock went 2556, 2542, 2544, 2548, 2561 and
    2563 within two hours.
- **Decision:**
  - A rise that new cases account for is recorded as the old lock plus the new cases'
    visibly-off cells (`visible_parity::judge`, `Judgement::Record`), not as the run's count.
  - A drop (`--lower-visible-lock`) still records the count. So does a rise beyond the new
    coverage under `--allow-regressions`.
- **Consequences:**
  - A probe PR's lock goes stale only when another lock writer merges first. That is the 0093
    case of two probe PRs conflicting on the file, where the fix is still to take `main`'s side and
    re-run `make eval-baseline`.
  - The ratchet is no looser than on `main`. The PR adds exactly its new cases' cells to the
    allowance main already had, and the steward's daily lowering still takes back the slack.
