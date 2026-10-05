# 0021 — Keep a buffer of ready sonnet tasks

- **Status:** accepted (2026-10-05, owner request; T-0032)
- **Context:** Decision 0018 split the workers by model tier. The sonnet worker is the cheap,
  frequent one (hourly, up to 3 tasks a run) but cannot plan: only the opus worker (when it has no
  opus task) and the daily steward (when a lane falls below 3 or 2 ready tasks of *any* tier)
  run the gap-triager or planner. Nothing watched the sonnet queue, so it could run dry while
  opus work piled up, leaving the cheaper capacity idle.
- **Decision:**
  - Target: **at least 10 ready, unclaimed `tier: sonnet` tasks** across lanes, measured with
    `python3 scripts/tasks.py ready --tier sonnet` (same filter as `next`).
  - The **opus worker** checks it after merging its task (work-loop step 6.5) and, when below the
    target, runs the gap-triager (functions) and/or the planner for the current milestone, asking
    for sonnet-sized tasks, until the count reaches 10–15 or no sonnet-sized work remains.
  - The **steward** does the same as part of its queue step (agent-workflow.md, which its routine
    prompt already reads), in addition to the per-lane thresholds in its prompt.
  - Refills follow the tier rubric (docs/process/task-format.md): work is *cut* into sonnet-sized
    pieces where the rubric allows; a task is never labelled `sonnet` to hit the number.
- **Consequences:** opus runs spend some time planning, which is the better use of the expensive
  tier; the sonnet worker rarely finds nothing to do. Routine prompts are unchanged.
