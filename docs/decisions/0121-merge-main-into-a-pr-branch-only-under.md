# 0121 — Merge main into a PR branch only under the merge lock unless it conflicts

- **Status:** accepted (2026-10-25, T-2751). It amends 0075 and 0028; 0020 and 0053 are unchanged.
- **Context:** Over the 48 h up to 2026-10-25, PRs made 383 "merge origin/main" commits, about two
  per PR. Each one starts a full CI run of about 7.7 minutes, so these runs used more than half of
  all CI minutes. 190 of those merges were made before the PR took the merge lock. `main` moved
  again before the lock was taken, so the merge was made a second time under the lock. Under 0075
  the lock holder already merges `main` in (steward skill "Merge lock", step 2), and with the lock
  held the up-to-date check of 0028 passes on the first green CI. A merge of `main` made before the
  lock therefore buys nothing, unless the branch conflicts. The texts still said otherwise: 0028
  and the steward skill's check (c) said "merge `main` in, run `make check`, push", and the
  work-loop's step 5c merged `main` in on resume.
- **Decision:** outside the merge lock, an agent merges `origin/main` into a PR branch only for one
  of these reasons:
  1. a textual conflict: the PR's `mergeable_state` is `dirty`, or a local test-merge of
     `origin/main` conflicts. This is 0020's rule, unchanged: GitHub runs no CI on a `dirty` PR, so
     the conflict is resolved at once, never waited out;
  2. a known semantic collision, the kind 0028's check exists to catch (0016: allocation is best
     effort): `main` now holds a task id or decision number that the branch also adds, as found by
     `tasks.py validate` or `check_decisions.py` failing on a local test-merge, or by red CI on the
     merged head under the lock. The PR merges `main` in and renumbers its own file.
  A branch that is only behind `main` is not updated. That happens once, under the lock (steward
  step 2). Check (c) of the merge policy and 0028's mechanical check are unchanged: they hold the
  merge itself, and the lock holder brings `main` in when the check fails.
  The same rule applies to orphaned PRs (0050) and to a branch resumed after the review-round limit
  (0095). The dormant merge-queue text (0053) already says the same thing and stays as written.
- **Consequences:** a PR normally makes one merge of `main`, under the lock, and gets one CI run for
  it. That should save about half of the CI minutes the 383 merges cost. A PR's CI before the lock
  runs against the `main` of its last push. A clash with newer work that does not show as a textual
  conflict (an id collision, or two changes that break each other) is found by the CI run under the
  lock, as before. The PR then releases the lock, fixes the clash and takes the lock again, so it
  never lands on `main`. Not merging early saves CI minutes, at the cost of finding a late clash a
  little later. The routine prompts already defer to the steward skill ("branch up to date with
  main" is still a merge condition), so they are unchanged.
