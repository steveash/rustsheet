# 0194 — Steward merges stranded ready-to-merge PRs in shadow mode

- **Status:** accepted (2026-10-31, T-3344). It amends 0050 (orphaned agent PRs) and 0157 (the
  merge-train modes) for `shadow` mode only.
- **Context:** In `shadow` mode a worker marks its PR `ready-to-merge` and then must merge it
  itself under the merge lock. On 2026-10-30 the lock was held 86% of the day, its wait p90 was
  54 min, and the queue reached 15 waiters. Workers' 110-minute sessions ended in the queue, and
  their PRs stayed ready but unmerged: eight on 2026-10-30, idle 4 to 13 h, and seven more on
  2026-10-31, idle 9 to 21 h. Both batches were merged by hand.

  Nobody else picks such a PR up:
  - 0050's orphan rule says a PR "whose worker already marked it ready, on its current head, is
    not orphaned";
  - only a train merges ready PRs, and in `shadow` mode no train merges anything;
  - the merger routine is an owner step that has not happened.

  The steward's 2026-10-31 run also could not list the stranded PRs: it stopped `make
  merge-train-stats` after 90 s and blamed `gh` authentication. In fact `gh api` works there
  through the session proxy (`gh auth status` misreports it). The full report simply takes
  minutes, because it reads every run, job and PR of the window. `stranded()` alone takes about
  5 s.
- **Decision:**
  1. **The sweep.** In `shadow` mode the steward merges a `ready-to-merge` PR that has been
     stranded past `STRANDED_AFTER` (2 h, `scripts/merge_train_stats.py`), by the per-PR flow
     under the merge lock, when all of these hold:
     - its verdicts hold on its current head: `make merge-ready-verify PR=<n>` passes, or the
       steward runs the applicable reviewers itself on that head, as 0050 requires, and marks
       it ready with those verdicts;
     - CI is green on the head;
     - no review thread asks for changes without a later resolution;
     - the task is not `status: blocked`, and the task file is `status: done` with an Outcome.

     Merging `main` in under the lock moves the head. The same verdicts still stand, as for any
     worker in `shadow` mode, and CI must pass on that new head.
  2. **Finding the stranded PRs.** The steward runs `make merge-train-stats STRANDED=1`, which
     prints only the stranded-PR line in seconds. If even that cannot run, it builds the same
     list with the GitHub tools, by `stranded()`'s definition: open PRs labelled
     `ready-to-merge`, not labelled `merge-train-ejected`, whose newest merge-ready marker comment
     is older than `STRANDED_AFTER`. A PR with no marker is not on the list.
  3. **Reporting.** The daily report names each PR the sweep merged.
  4. **`lock` and `train` modes are unchanged.** In `lock` mode nothing is marked ready, and in
     `train` mode a stranded PR still triggers the emergency flip (0157).
- **Consequences:** a stranded PR now waits at most until the next steward run (05:47 or 18:47
  UTC) instead of indefinitely. The merger routine, then `train` mode (T-2991), remains the real
  fix: a train merges ready PRs as they arrive. SPEC-0027 is unchanged. Its "stranded"
  definition is the one the sweep uses, and this record is where the sweep is defined.
