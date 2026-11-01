# 0182 — tasks.py validate rejects a new task that lists cells another open task lists

- **Status:** accepted (2026-10-30, T-3229; the owner asked for the check to be built)
- **Context:** Parallel gap triages fill the queue from the same eval report, and none of them searched the
  open tasks before filing. On 2026-10-30, review of four idle refill PRs found that 5 of their 15 new
  tasks duplicated open tasks on main:
  - T-2915 duplicated T-2928;
  - T-3159 duplicated T-1749;
  - T-3160 duplicated T-3070, T-2561, T-3013 and T-2638;
  - T-3194 duplicated T-3217;
  - T-2859 duplicated T-2724.

  Each duplicate cost a review round. Had one merged, two tasks would have split one fix.
- **Decision:** `scripts/tasks.py validate` fails when a task the branch adds shares an eval cell with
  another open task and neither file names the other's id.
  - "Added" means a task file absent on the merge base of origin/main and HEAD.
  - Cells are read from cell-list lines (docs/process/task-format.md) and from inline citations of
    eval-suite cases.
  - Naming the other task records a deliberate split, and reviewers judge whether the split is real.
  - With no origin/main merge base, the check prints a warning and is skipped. Merge-train id checks
    run `validate --root` in trees that have none.
- **Consequences:**
  - A triage that files cells an open task already owns must do one of three things: fold the work
    into that task, drop the shared cells, or name the owner in both files.
  - Tasks already on main are not re-checked, so existing splits stay valid.
  - The check is a heuristic, so it reduces review effort but does not replace it. A passing mention
    of the owner counts as a split, so T-3159 and T-3160 would still pass. A branch cut before its
    duplicate merged is caught only on CI's merge commit or after a catch-up merge, as with T-3194.
  - `stats` stays git-free.
