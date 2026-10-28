---
id: T-2996
title: "Merge-train policy record and routine prompts in routines.md"
milestone: M2
lane: quality
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-2995]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
The merge train's policy is on record, and the routines' prompts in `docs/process/routines.md`
match it. Nothing changes in behavior until the switch moves off `lock`.

## Context
- The text of the record comes from SPEC-0027 "Decision records and governance": "Merges go
  through the merge train when the switch says so". It amends 0028 (the guarantee is detective,
  intermediate states are accepted, readiness is an attestation, strict mode is withdrawn), 0075,
  0050 (in `lock` mode, ready PRs may be merged or unlabelled at once) and 0121, and records the
  switch rules. The record cites the spec's sections rather than copying them.
- The switch rules (SPEC-0027 "The switch"):
  - the mode is read from `origin/main`;
  - a PR whose only change sets `mode = "lock"` may be squash-merged by any agent under the fair
    lock, with green `hygiene` and no reviewers;
  - `check_governance.py` accepts exactly that diff without a decision record;
  - the flips to `shadow` and `train` stay fully gated.
- routines.md gains:
  - the merger row (Sonnet, `5,35 * * * *`);
  - the merger prompt (SPEC-0027's draft);
  - the worker and steward clauses (SPEC-0027 "Worker and steward prompt changes").

  routines.md becomes the only home of the prompt, and this PR replaces the spec's copy with a
  pointer.

## Acceptance criteria
- [ ] The decision record lands with the amendments above, and 0028, 0050 and 0075 carry
      "Amended by" notes.
- [ ] `check_governance.py` accepts a diff whose only change is `mode = "lock"` in
      `scripts/merge_train.toml` and refuses every other change without a record. Tests cover
      the boundary cases: an extra file, an extra line, `mode = "train"`.
- [ ] routines.md holds the merger row and prompt, and the worker and steward clauses. SPEC-0027's
      prompt block is replaced with a pointer, and any wording change is made in the spec as
      well.
- [ ] The owner steps of SPEC-0027 are listed under "Needs owner" in docs/status.md.
- [ ] `tasks.py validate`, `check_decisions.py`, `check_docs_readers.py`, `make typos` and the
      scripts unit tests pass. The architecture and security reviewers approve.

## Out of scope
- Skill and process texts, and setting `shadow` (T-2989). Creating routines, which only the owner
  can do.
