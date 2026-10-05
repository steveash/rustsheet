# 0022 — Parallel functions lanes and looping worker sessions

- **Status:** accepted (2026-10-05, owner request; T-0034)
- **Context:**
  - Most remaining work (M2, M5, M6, M9, M10) is function fixes, all in one `functions` lane. With
    one task in flight per lane, function work ran one task at a time however many workers ran.
  - The opus worker stopped after one task ("complete exactly one task end to end") and then sat
    idle for up to an hour; the sonnet worker stopped after 3 tasks or 45 minutes.
  - Starting a cloud session from scratch is slow, so short runs pay that cost often.
  - The "lane is busy" rule lived only as prose in the routine prompts.
- **Decision:**
  - Split `functions` into nine lanes by module: `functions-math`, `-stat`, `-criteria`,
    `-financial`, `-engineering`, `-text`, `-lookup`, `-date`, `-logic` (table in
    agent-workflow.md). Plain `functions` stays valid on done/dropped tasks and as the
    `next-task`/`ready-tasks` filter for "any functions-* lane"; `validate` rejects it on open
    tasks and `new` refuses it. The 34 open functions tasks were relabelled by module (33 into
    `functions-*` lanes, T-0920's cross-cutting ports into `engine`).
  - `make next-task` / `ready-tasks` enforce one task in flight per lane: a lane is busy while a
    pushed `agent/<lane>/*` branch has a commit in the last 2 hours, unless it is an unresumed
    handoff (decision 0018), which the opus worker may still resume, or its task is already
    `done` on main (merged, branch not yet deleted).
  - Both worker prompts loop: after each merge they pick the next task, until 100 minutes into
    the session (at most 6 tasks). Both run every 2 hours, staggered by an hour; an optional
    second sonnet worker (same prompt, odd hours) can be added when usage allows.
- **Consequences:**
  - Several workers can fix functions at once; fixes spanning families take their main module's lane.
  - Fewer cold starts per task. A run may overlap the next scheduled run of another worker; the
    busy-lane check keeps them apart.
  - Baseline and status-log conflicts may rise with parallelism; T-0028/decision 0020 resolve
    conflicts proactively, and the status log is handled separately (T-0033).
  - The owner must re-apply the routines (prompts and schedules) when this merges.
