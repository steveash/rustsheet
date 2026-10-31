# 0204 — In train mode the merge lock refuses per-PR claims

- **Status:** accepted (2026-10-31, T-3476). It enforces what 0157 and the `work-loop` skill
  already say for `train` mode ("do not take the lock"), in `scripts/merge_lock.py`.
- **Context:** The repo went to `train` mode at 10:20 UTC on 2026-10-31 (T-2991, #2547). In the
  7 hours after, the lock's push history shows about 12 train holds and 27 per-PR holds: 23 from
  other sessions and 4 from the session that filed this record.

  They came from long-running interactive sessions and worker runs started before the switch,
  which kept the per-PR flow. One unidentified session made 7 such merges. Each per-PR hold lasts
  a full CI run, and the FIFO queue put trains behind them. 13 PRs marked `ready-to-merge` were
  stranded past 2 h, which is the `train`-mode trigger for the emergency flip.

  The worker routines' prompts were already mode-aware. The gap was sessions whose instructions
  predate the switch, and nothing in the lock checked the mode.
- **Decision:** `merge_lock.py acquire --pr N` and `wait --pr N` exit 5 (`REFUSED`) in `train`
  mode, with a message naming `make mark-ready`.
  - **Exceptions:**
    - a PR labelled `merge-train-stop`, the emergency-flip PR, the one per-PR merge `train` mode
      allows (`steward` skill, "Emergency flip to `lock`" step 4);
    - `--train` claims;
    - `release` and `status`.
  - **The mode:** `make merge-mode`'s read (`merge_train.read_mode`, a freshly fetched
    `origin/main`), checked once per call before the first attempt.
  - **Failure handling:** a mode that cannot be read reads as `lock`, and a PR whose labels
    cannot be read is let through. A GitHub or git outage therefore never blocks an emergency
    flip; at worst it lets one per-PR claim through, which is today's behaviour.
- **Consequences:**
  - A session that missed the switch gets a clear refusal instead of queueing ahead of the
    trains.
  - The steward's "Merge lock" and the work-loop's `train` bullet name exit 5.
  - `lock` and `shadow` modes are unchanged.
  - A claim made before the switch keeps its place until it is released or renewed; a renewal
    is a new `acquire --pr`, so it is refused too.
  - The merger routine (an owner step) is still what makes trains run on a schedule. This record
    only removes the per-PR traffic in their way.
