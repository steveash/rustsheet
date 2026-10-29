# 0164 — The merge-train switch goes to shadow; skills describe each mode

- **Status:** accepted (2026-10-29, T-2989). It carries out decision 0157's flip to `shadow`
  (SPEC-0027 "Rollout", row 10) and leaves the flip to `train` to T-2991.
- **Context:** Decision 0157 set the merge policy for each value of the switch, but the skills
  and the process doc still described only the per-PR flow under the lock, and the switch said
  `lock`. The merge train's slices (T-2985 to T-3040) are in place; the shadow merger needs PRs
  marked ready to have candidates, and T-2991 needs at least 24 h of shadow evidence.
- **Decision:**
  - `scripts/merge_train.toml` says `mode = "shadow"`.
  - The `work-loop` skill (step 6.3), the `steward` skill (merge policy condition (c), a "Merge
    train" section, orphaned PRs, the daily thresholds) and `docs/process/agent-workflow.md`
    (loop step 9, the merger role) describe the flow for each mode of `make merge-mode`, so a
    later flip changes the flow with no other text edit:
    - `lock`: the per-PR flow under the fair lock, unchanged;
    - `shadow`: `make mark-ready`, then the per-PR flow; the worker removes `ready-to-merge`
      after its own merge;
    - `train`: `make mark-ready` and move on; the 45-min stall fallback is the safety net.
  - Both skills state the emergency flip to `lock` and the revert-PR handling exactly as
    SPEC-0027 and decision 0157 do.
- **Consequences:**
  - In `shadow` mode nothing merges through a train; workers still merge every PR themselves
    under the lock. Marking costs one comment and one label per PR.
  - SPEC-0027's daily thresholds trigger the emergency flip only in `train` mode. In `shadow` mode
    the steward only reports the numbers: nothing merges through a train there, so a flip would
    change nothing, and the numbers are T-2991's evidence.
  - Owner step 3 (move the worker and steward routines to routines.md's texts) is now due.
  - Rollback: a flip PR to `lock` (decision 0157), or a revert of this record and its texts.
