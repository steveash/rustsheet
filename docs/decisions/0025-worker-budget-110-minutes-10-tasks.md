# 0025 — Worker session budget: 110 minutes, 10 tasks

- **Status:** accepted (2026-10-05, owner request; T-0037)
- **Context:** decision 0022 made both workers loop until 100 minutes into the session or 6
  started tasks. The owner wants each 2-hour slot used as fully as possible, by the opus worker
  as well as the sonnet one. The work-loop skill also still said the opus worker "completes
  exactly one task per run" (left from decision 0018), and a blocked or handed-off task said
  "stop", which a worker could read as ending the run.
- **Decision:**
  - Both worker prompts (docs/process/routines.md) keep taking tasks until 110 minutes into the
    session or 10 started tasks, whichever comes first. They then start nothing new and push
    what is in flight.
  - The work-loop skill says: after a merge (and any queue top-up), go back to step 1 while the
    budget allows. Blocking or handing off a task ends work on that task only, not the run.
- **Consequences:**
  - A run can finish in-flight work after the 2-hour mark, overlapping the next run of the same
    worker. That is safe: the in-flight branch holds its claim and makes its lane busy
    (decision 0022), so the next run picks other work.
  - The owner must re-apply the two worker prompts from routines.md for the new budget to take
    effect. The skill change takes effect on merge.
