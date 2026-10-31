---
id: T-3344
title: "Steward merges ready-to-merge PRs stranded in shadow mode"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
In `shadow` mode a PR that its worker marked `ready-to-merge` and then left behind still gets
merged: the daily steward run merges every such PR that has sat idle for more than 2 hours, by
the same per-PR flow under the merge lock that its worker would have used.

## Context
On 2026-10-30 eight PRs were stranded like this for 4 to 13 hours, all green, approved and marked
ready: #2344, #2350, #2358, #2360, #2371, #2374, #2377 and #2386. The steward's 2026-10-30 health
run counted 7 of them. They were merged by hand.

Why they strand:
- In `shadow` mode a worker marks its PR ready and then must merge it itself under the merge lock
  (work-loop step 3; `steward` skill "Merge train").
- The lock was held 86% of the day, its wait p90 was 54 min, and the queue reached 15 waiters.
  Each holder keeps it for one CI run (about 8-10 min).
- A worker's session lasts 110 minutes, so it often ends before its turn and leaves the PR
  marked ready but unmerged.
- Nobody picks such a PR up:
  - the `steward` skill's orphaned-PR rule says "A PR whose worker already marked it ready, on
    its current head, is not orphaned";
  - only a train merges ready PRs, and the merger routine is not yet running (status.md "Needs
    owner", owner step (1)).

The gap closes for good when `train` mode is on (T-2991). Until then, the steward must sweep these
PRs. In `lock` mode nothing is marked ready, so the sweep is a no-op there.

Note: the edit touches `.claude/skills/steward/SKILL.md`, a protected path, so it needs the
decision record the governance gate asks for.

## Acceptance criteria
- [ ] The `steward` skill (orphaned-PR rule or "Merge train" section) says that in `shadow` mode
      a `ready-to-merge` PR whose head has not moved for 2 h is merged by the steward per the
      per-PR flow. That flow is:
      - wait for the merge lock;
      - merge `origin/main` in, regenerating baselines on a baseline-only conflict;
      - wait for CI green on that head;
      - run the up-to-date check;
      - squash-merge with `expectedHeadSha`;
      - remove the label.

      The review table must still show APPROVE from every applicable reviewer on the PR. In
      `train` mode the rule does not apply, because the train merges them.
- [ ] The steward's daily report names each PR it merged this way.
- [ ] `make merge-train-stats`' "stranded" count is the input the steward acts on, or the skill
      says how the steward finds these PRs.
- [ ] A decision record covers the steward skill change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Turning on `train` mode (T-2991) and the merger routine (owner step).
- Making the merge lock faster.
