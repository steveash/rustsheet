# 0050 — Steward merges orphaned agent PRs that are green, reviewed and idle

- **Status:** accepted (2026-10-12, owner decision). It extends the `steward` skill's merge
  policy, keeps decision 0028's up-to-date gate for every merge, and reuses decision 0020's
  "merge `main` into another session's stale agent PR" step. Amended by 0053: with the merge
  queue on, enqueue instead of the up-to-date check.
- **Context:**
  - Worker sessions stop on a time budget (decision 0025). Some stop with a PR open that is
    finished or nearly finished.
  - The steward routine's step (4) says "do not merge PRs opened by another session", and the
    skill lets only the opening agent merge. So nobody merges these PRs, and their tasks stay
    claimed and undone.
  - About 8 such PRs were open on 2026-10-12. #330 (T-0411) is 5 days old, green and approved,
    and the steward has already merged `main` into it (docs/status.md). #517 and #586 both
    implement T-0298.
- **Decision:**
  - **Orphan rule.** The steward may squash-merge a task PR opened by another session (branch
    `agent/<lane>/T-XXXX-*`) when all of these hold:
    - It has been open 24 h or more, and its branch has not moved for 2 h or more, so no session
      is working on it. "Moved" is measured by push time (the latest head's first CI run, or the
      branch's last push event), not by commit date. The steward's own commits (merges of `main`,
      bookkeeping, identified by its `Claude-Session` trailer) do not reset this clock.
    - It is not a draft, and its task is not `status: blocked`.
    - CI is green on the latest head.
    - The branch passes decision 0028's up-to-date gate. The steward may merge `main` into the
      branch first.
    - The steward has run the `review` skill's applicable reviewers itself on the current diff, at
      the head it merges, and each says APPROVE; no review thread or comment asks for changes
      without a later resolution. It does not trust the PR body's agent-review table, which names
      no head sha and may predate later commits or never have been run (architecture review).
    - The task file on the branch is `status: done` with an Outcome section. If it is not, the
      steward either finishes that bookkeeping (and waits for CI on the new head) or leaves the PR
      alone.
  - **Duplicates.** When two open PRs implement the same task id, the steward picks the survivor:
    the one that is green and reviewed or, only if neither is, the newer one. It closes the other
    (a comment linking the survivor; nothing is deleted) only after the survivor has merged, and
    only when the other's branch has not moved for 2 hours, so a session still pushing to it is
    never cut off. The 24 h age does not apply to closing a duplicate.
  - **Otherwise.** If any condition fails, today's behaviour stays: fix CI, comment the blocker,
    or mark the task blocked with reasons.
  - The rule is written in `.claude/skills/steward/SKILL.md`. It is not copied into other files.
- **Consequences:**
  - Finished work from expired sessions lands within a day, and its task stops blocking the lane.
  - Every merge still goes through the same gates as a worker merge: green CI on the merged head,
    decision 0028's ancestor check and `expectedHeadSha`, and reviewer approval. The only new
    actor is the steward.
  - The 2 h idle window matches `tasks.py`'s `BUSY_SECONDS`, the worker prompts' resume rule, and
    the `expectedHeadSha` guard. So the steward does not merge under a session that is still
    pushing, and a push that races the merge makes the merge call fail.
  - A closed duplicate keeps its branch. Its task is done once the survivor merges, so
    `make next-task` will not offer the task again.
  - The live steward routine still says "do not merge PRs opened by another session". Until the
    owner replaces that step (below), a routine run follows its prompt. Agents never edit
    routines or the prompts in docs/process/routines.md (decision 0008).

## Owner action

The steward routine's current step (4), quoted verbatim:

> for agent PRs open longer than 24 h: do not merge PRs opened by another session; instead fix CI on them per the `steward` skill, comment the blocker, or mark their tasks blocked with reasons

Replace it in the steward routine with the text below. This PR already updates the "Prompt —
steward (daily)" block of docs/process/routines.md to match (decision 0008 keeps them identical):

```
for agent PRs open longer than 24 h: merge a PR opened by another session only when it meets every condition of the `steward` skill's "Orphaned agent PRs" rule (decision 0050), close duplicates per that rule, and otherwise fix CI on them per the `steward` skill, comment the blocker, or mark their tasks blocked with reasons
```
