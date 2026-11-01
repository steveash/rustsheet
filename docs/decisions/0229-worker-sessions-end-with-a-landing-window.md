# 0229 — Worker sessions end with a landing window

- **Status:** accepted (2026-11-01, T-3684; the repository owner asked for this change in session
  https://claude.ai/code/session_017265sMr6AAKLLZpHYUvwh6). Amends decision 0025.
- **Context:**
  - Decision 0025 lets a worker start tasks until minute 110 of its session. A task started late
    is usually still in CI or review when the session ends, and nobody else picks it up until an
    orphan rule or a resume rule fires hours later.
  - At 07:05 UTC on 2026-11-01 the open PRs included 8 ejected from the merge train (idle 1 to
    20 h) and about 10 non-draft PRs never marked ready (idle 10 to 14 h). A ready PR is safe,
    because the merger, which runs every 30 minutes, owns it. These two groups are not.
- **Decision:**
  - Both worker prompts start new tasks only until minute 80 of the session (still at most 10
    tasks). From then until minute 110 the worker lands the PRs it opened, by the work-loop
    skill's step 6.6: it finishes reviews and marks unmarked PRs, and it clears ejected PRs (with
    `make catch-up` for a baseline-only conflict, decision 0207).
  - A PR that is still unmarked or ejected at minute 110 gets `## Landing notes` on its task file
    (a PR comment for a task-only PR). They record its head, its state (CI job and run, reviewers
    run and missing, or the ejection reason), the next step, and what not to repeat, so the run
    that resumes it starts from facts.
- **Consequences:**
  - Each session authors fewer tasks and leaves fewer half-landed PRs behind.
  - The owner must re-apply the sonnet and opus worker prompts from docs/process/routines.md for
    the new budget to take effect. The skill change takes effect on merge.
  - Merge-train repairs that need judgment still wait for a resume rule or the janitor of
    T-3630. Turning them into queued repair tasks, with the landing notes as their context, is
    later work.
