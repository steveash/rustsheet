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
The merge train's policy is on record. The routines' prompts in `docs/process/routines.md` match
it, and the emergency flip to `lock` is mechanically checkable. Behaviour does not change until
the switch leaves `lock`.

## Context
- The record's text: SPEC-0027 "Decision records and governance", "Merges go through the merge
  train when the switch says so". It amends:
  - 0028: the guarantee is detective; intermediate states are accepted; readiness is an
    attestation; the strict-mode recommendation is withdrawn;
  - 0075;
  - 0050: in `lock` mode, ready PRs may be merged or unlabelled at once;
  - 0121.

  It also records the switch rules. It cites the spec's sections rather than copying them.
- The switch rules (SPEC-0027 "The switch"):
  - the mode is read from `origin/main` and fails closed;
  - the flip rule:
    - `git diff --name-status --no-renames` is exactly `M scripts/merge_train.toml`, with mode
      `100644` on both sides;
    - both blobs parse with `tomllib` and are identical except that the top-level `mode` becomes
      `"lock"`;
  - `make merge-flip-check PR=<n>` applies the same rule through the GitHub files API and prints
    the head sha. The merging agent merges with `expectedHeadSha`;
  - `check_governance.py` waives only the decision-record rule for a passing diff. The
    test-deletion and probe-tag checks still run;
  - the P0 task goes in a separate PR, and the `merge-train-stop` label is used;
  - a revert PR that passes `make merge-revert-check` (T-2994) is also merged without reviewers;
  - the flips to `shadow` and `train` stay fully gated.
- routines.md gains:
  - the merger row (Sonnet, `5,35 * * * *`);
  - the merger prompt (SPEC-0027's draft);
  - the worker and steward clauses (SPEC-0027's drafts; the 45-min fallback is `train` mode
    only).

  routines.md becomes the only home of these texts. This PR replaces SPEC-0027's draft blocks
  with a pointer to routines.md.

## Acceptance criteria
- [ ] The decision record lands with the amendments above, and 0028, 0050 and 0075 carry
      "Amended by" notes.
- [ ] `make merge-flip-check` and `check_governance.py`'s waiver implement the flip rule. Tests
      accept the one-line `lock` change and refuse:
      - an extra file, an extra line;
      - `mode = "train"` or `"shadow"`;
      - a symlink or a type change;
      - a duplicate key, a nested `mode`;
      - a rename;
      - a head that moved after the check.

      They also show that the waiver silences only the decision-record rule.
- [ ] routines.md holds the merger row and prompt, and the worker and steward clauses.
      SPEC-0027's draft blocks are replaced with a pointer to routines.md.
- [ ] The owner steps of SPEC-0027 are listed under "Needs owner" in docs/status.md. Step 3 (the
      worker and steward routines) is marked "after T-2989 merges".
- [ ] `tasks.py validate`, `check_decisions.py`, `check_docs_readers.py`, `make typos` and the
      scripts unit tests pass. The architecture and security reviewers approve.

## Out of scope
- Skill and process texts, and setting `shadow` (T-2989). Creating routines, which only the owner
  can do.
