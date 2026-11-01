# 0208 — The owner re-flips the merge train to `train` after the stranding flip (owner ruling)

- **Status:** accepted (2026-10-31, T-3488; owner ruling in session
  https://claude.ai/code/session_018kHzCtGoS4gXMYKAZWDHEd: "i want to fix this right now", then
  chose "flip and drain" when asked whether to re-flip to `train` and drain the ready PRs).
  *Amended by 0224: a stranded PR no longer triggers the emergency flip; it escalates (a merger
  pass, the janitor, a P0), and only a correctness failure flips at once.*
- **Context:**
  - The steward's emergency flip to `lock` (#2613, 19:17 UTC) followed SPEC-0027's stranded-PR
    threshold: 10 PRs marked ready more than 2 h earlier. Decision 0192 requires a re-flip to
    meet SPEC-0027's full bar again or to have its own owner ruling.
  - The stranding had four causes, all found on the first live `train` day:
    - until #2589 (18:25) the merger stopped at step 7.2 on every stale GitHub test merge, so no
      train merged anything (decision 0200);
    - a batch took only the 5 oldest ready PRs, so ejections shrank it (decision 0206);
    - an ejected PR waited for its ended session even when only baseline lists conflicted
      (decision 0207);
    - per-PR merges kept taking the lock in `train` mode, so the train queued behind them for
      31 min and more (decision 0204, #2605: the lock now refuses those claims in `train` mode).
  - After the flip, 25 PRs stayed labelled `ready-to-merge`; in `lock` mode each would take one
    serial lock turn.
- **Decision:** `mode = "train"` again, by owner ruling, with all four fixes on `main`. The owner
  asked to drain the ready PRs at once: a session may take over the lock from a per-PR holder
  and run train passes back to back until the ready set is empty.
- **Consequences:**
  - Workers go back to `make mark-ready`; `make merge-lock` refuses their per-PR claims.
  - T-3488 stays open to confirm, from `make merge-train-stats STRANDED=1`, that no PR strands
    again; the steward's emergency flip still applies if one does.
