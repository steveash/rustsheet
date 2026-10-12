# 0048 — A task blocked on its pushed branch stays claimed at any age

- **Status:** accepted (2026-10-12, T-1414)
- **Context:**
  - A worker that blocks a task records `status: blocked` and the "Blocked because" section on its
    branch and opens a draft PR (work-loop step 5). Main keeps `status: todo` until that PR merges,
    which a blocked draft never does.
  - `branch_claims` in `scripts/tasks.py` counts a pushed branch as a claim only for 24 h
    (`CLAIM_TTL_SECONDS`), and handoffs at any age. After a day a blocked task looks ready again.
  - On 2026-10-12 `make next-task LANE=functions TIER=opus` returned T-1186 (blocked on #436; its
    research task T-1187 closed as "not found"), and the opus ready list held T-1155 (blocked on
    #409, waiting on T-0916). A worker following the queue would restart work whose blocker holds.
- **Decision:** a task whose file on a pushed `agent/*` branch says `status: blocked` is claimed at any
  branch age. It still frees its lane (decision 0022). Unblocking such a task means resuming its branch
  (set `status: todo` or `in-progress` there, per the unblocking rule in
  docs/process/agent-workflow.md) or deleting the branch, not editing the task on main.
- **Consequences:**
  - `make next-task` and `make ready-tasks` no longer offer tasks that are blocked on a branch.
  - A blocked task becomes visible again only through its branch: the steward's blocked-task check
    reads the branch, and deleting an abandoned blocked branch releases the task.
  - Stale non-blocked branches (abandoned or merged but not deleted) still stop claiming after 24 h.
