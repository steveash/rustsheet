---
id: T-3761
title: "The merge train files a repair task with its evidence when it ejects a PR"
milestone: M2
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
An ejected PR whose fix needs judgment becomes queued work: the train files a repair task that
`make next-task` hands out before new work, carrying the evidence a resumer needs.

## Context
Owner request, 2026-11-01 (session https://claude.ai/code/session_017265sMr6AAKLLZpHYUvwh6).
Ejected PRs strand because only the PR's worker may fix them, and workers' sessions end. The
janitor (T-3630, decision 0224) clears mechanical ejections and posts an orphan note after 4 h;
`ci-red`, real conflicts and validate failures still wait on idle-branch resume rules.
What the repair task carries, recorded by the script at ejection time (facts, not guesses):
- PR, head and base shas, and the ejection class;
- `ci-red`: the failing job, the run URL, a bounded log tail, the bisection result;
- conflicts: the conflicting files (`git merge-tree`);
- validate failures: the `FAIL` lines;
- links to the original task file, the verdict records, and any `## Landing notes`
  (decision 0229).
The *why* of the change lives in the task file, PR body and landing notes. The repair task must
say that when main has changed the PR's premise (e.g. main already fixes the same cells another
way), the right outcome is to close the PR as superseded or block it with a note, not to force
a fix.
This changes SPEC-0027 (the ejection step) and the work-loop and steward skills, so it starts
with a short spec amendment and decision record.

## Acceptance criteria
- [ ] A decision record and a SPEC-0027 amendment define the repair task's fields, who files it
      (the train, through step mode), its priority, and how `next-task` orders it.
- [ ] The train files the task (a task-only PR, decision 0231) for `ci-red`, real conflicts and
      validate failures; mechanical ejections stay with the janitor.
- [ ] Tests with fixture ejections of each class.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Waking the original session to repair its own PR (a separate probe).
