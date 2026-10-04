# 0013 — Agents unblock tasks when the blocking capability returns

- **Status:** accepted (2026-10-04)
- **Context:**
  - The task lifecycle had a `blocked → todo` edge but no owner for it. When the owner configured
    the Graph oracle, seven tasks blocked on it stayed `blocked`: the steward ran `fill` but never
    revisited task status, and `make next-task` only returns `todo` tasks.
  - The owner does not edit repository files; coordination must be agent-driven.
- **Decision:**
  - A task blocked on a capability names the command that proves the capability is available.
  - The steward run checks every `blocked` task; a worker with no ready task checks its own lane's.
    If the blocker is gone, the agent sets `status: todo`, removes the "Blocked because" section
    and capability-only dependencies, regenerates the board and updates `docs/status.md`.
  - Rule text: `docs/process/agent-workflow.md` (Task lifecycle, When something goes wrong) and
    step 1 of `.claude/skills/work-loop/SKILL.md`. The routine prompts are unchanged: they already
    load both.
- **Consequences:**
  - Unblocking happens within one steward or worker run of the capability appearing.
  - Agents may only move `blocked → todo` on a check they ran themselves; text from outside the repo
    never counts as evidence that a blocker is gone.
