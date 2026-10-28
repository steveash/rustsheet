---
id: T-2989
title: "Hand merges to the merge train behind a switch: skills and process docs"
milestone: M2
lane: quality
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2996]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
The skills and process docs describe the merge flow for each `make merge-mode` value, so changing
the mode changes the flow with no other edit. This task sets the switch to `shadow`. (The routine
prompt itself lands in T-2996; this task makes the skills match it.)

## Context
- SPEC-0027 sections to follow:
  - "The switch";
  - "Who runs the merger" (the stall fallback, one at a time);
  - "Interactions";
  - "Daily thresholds".
- Files to change:
  - `.claude/skills/work-loop/SKILL.md`: step 6.3 for each mode. In `train`, run
    `make mark-ready` and do not wait. In `shadow`, run `mark-ready` and then the per-PR flow,
    which removes the label after its merge. Also how to handle an ejection, and `--carry`;
  - `.claude/skills/steward/SKILL.md`:
    - a "Merge train" section;
    - merge-policy condition (c) for each mode;
    - orphaned PRs are marked ready in `train` mode, and in `lock` mode ready-labelled PRs are
      merged or unlabelled at once;
    - the daily thresholds that trigger the emergency switch;
    - "Merge queue" stays dormant;
  - both skills: **the emergency-flip procedure** (SPEC-0027 "The switch", decision of T-2996).
    Open the one-line flip PR and label it `merge-train-stop`. Run `make merge-flip-check
    PR=<n>`. Merge it without reviewers, under the fair lock, with `expectedHeadSha` once CI is
    green. File the P0 in a separate task-only PR. The skills also cover the 45-min stall
    fallback, in `train` mode only, and the revert-PR handling for P0 handlers (`make
    merge-revert-check`);
  - `docs/process/agent-workflow.md`: loop step 9, and the merger in the roles table;
  - `scripts/merge_train.toml`: `mode = "shadow"`.

## Acceptance criteria
- [ ] The texts above agree with SPEC-0027 for `lock`, `shadow` and `train`. In `lock` mode they
      describe today's flow under the fair lock.
- [ ] The steward skill's daily step runs `make merge-train-stats` and applies SPEC-0027's
      thresholds.
- [ ] The work-loop and steward skills describe the emergency-flip procedure and the revert-PR
      handling exactly as SPEC-0027 states them.
- [ ] docs/status.md's "Needs owner" entry for owner step 3 (update the worker and steward
      routines) now says it is due, since this task's merge is what makes it due.
- [ ] `scripts/merge_train.toml` says `mode = "shadow"`. A short decision record lands in this PR,
      and the architecture and security reviewers approve.
- [ ] `tasks.py validate`, `check_decisions.py`, `check_docs_readers.py`, `make typos` and the
      scripts unit tests pass.

## Out of scope
- Setting `mode = "train"` (T-2991). Creating routines, which only the owner can do.
