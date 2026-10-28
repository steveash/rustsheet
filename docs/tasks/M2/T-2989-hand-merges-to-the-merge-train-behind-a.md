---
id: T-2989
title: "Hand merges to the merge train behind a switch: skills, process docs and the merger routine prompt"
milestone: M2
lane: quality
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2987]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
The skills, process docs and routine prompts describe the merge flow for each value of `make
merge-mode`, so that changing the mode changes the flow with no other edit. This task sets the
switch to `shadow`.

## Context
- SPEC-0027 sections to follow:
  - "Who runs the merger" and "Interactions";
  - under Rollout: the merger routine prompt, and the clauses for the worker and steward prompts;
  - under "Decision records and governance": the text of the record "Merges go through the merge
    train when the switch says so" (it amends 0028, 0075, 0050 and 0121, and holds the emergency
    switch rule).
- Files to change:
  - `.claude/skills/work-loop/SKILL.md`: step 6.3 runs `make mark-ready` in `train` mode and does
    not wait, and the step says how to handle an ejection;
  - `.claude/skills/steward/SKILL.md`: a new "Merge train" section; merge-policy condition (c) for
    each mode; in `train` mode orphaned PRs are marked ready instead of merged; "Merge queue"
    stays dormant;
  - `docs/process/agent-workflow.md`: loop step 9, and the merger in the roles table;
  - `docs/process/routines.md`: the merger row and prompt, and the worker and steward clauses;
  - `scripts/merge_train.toml`: `mode = "shadow"`.

## Acceptance criteria
- [ ] The texts above agree with SPEC-0027 for `lock`, `shadow` and `train`. In `lock` mode they
      describe today's flow, under the fair lock.
- [ ] routines.md holds the merger routine row and its prompt, and the worker and steward clauses.
      The prompt is SPEC-0027's text; if it differs, this PR amends the spec to match.
- [ ] The owner steps from SPEC-0027 are listed under "Needs owner" in docs/status.md.
- [ ] The decision record lands in this PR. The architecture and security reviewers approve.
- [ ] `tasks.py validate`, `check_decisions.py`, `check_docs_readers.py`, `make typos` and the
      scripts unit tests pass.

## Out of scope
- Setting `mode = "train"` (T-2991).
- Creating routines, which only the owner can do.
