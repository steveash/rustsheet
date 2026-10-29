---
id: T-2996
title: "Merge-train policy record and routine prompts in routines.md"
milestone: M2
lane: quality
status: done
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
  - the P0 task goes in a separate PR;
  - the `merge-train-stop` label is honoured only on an open PR that passes `make
    merge-flip-check`. `make merge-mode` prints an honoured stop label;
  - revert PRs opened by the merger (T-2994) are reviewed and merged under the normal flow, with
    P0 priority. There is **no** review waiver and no governance waiver for them;
  - the flips to `shadow` and `train` stay fully gated.
- routines.md gains:
  - the merger row (Sonnet, `5,35 * * * *`);
  - the merger prompt (SPEC-0027's draft);
  - the worker and steward clauses (SPEC-0027's drafts; the 45-min fallback is `train` mode
    only).

  routines.md becomes the only home of these texts. This PR replaces SPEC-0027's draft blocks
  with a pointer to routines.md.

## Acceptance criteria
- [x] The decision record lands with the amendments above, and 0028, 0050 and 0075 carry
      "Amended by" notes.
- [x] `make merge-flip-check` and `check_governance.py`'s waiver implement the flip rule. Tests
      accept the one-line `lock` change and refuse:
      - an extra file, an extra line;
      - `mode = "train"` or `"shadow"`;
      - a symlink or a type change;
      - a duplicate key, a nested `mode`;
      - a rename;
      - a head that moved after the check.

      They also show that the waiver silences only the decision-record rule.
- [x] routines.md holds the merger row and prompt, and the worker and steward clauses.
      SPEC-0027's draft blocks are replaced with a pointer to routines.md.
- [x] The owner steps of SPEC-0027 are listed under "Needs owner" in docs/status.md. Step 3 (the
      worker and steward routines) is marked "after T-2989 merges".
- [x] `tasks.py validate`, `check_decisions.py`, `check_docs_readers.py`, `make typos` and the
      scripts unit tests pass. The architecture and security reviewers approve.

## Out of scope
- Skill and process texts, and setting `shadow` (T-2989). Creating routines, which only the owner
  can do.

## Outcome
- Decision 0157 is the policy record: the switch, 0028 (detective tree check, readiness as an
  attestation, strict mode withdrawn), 0075 (the merger holds the lock for a batch in `train`
  mode), 0050 (mark ready in `train` mode; merge or unlabel at once in `lock` mode), the flip
  rule, the stop label (implemented by T-3055), revert PRs under the normal flow. 0028, 0050 and
  0075 carry "Amended by 0157".
- `scripts/merge_flip.py` holds the flip rule. `make merge-flip-check PR=<n>` applies it to a PR
  (files API, fetched `refs/pull/<n>/head`, a second read of the head) and prints the head sha.
  `check_governance.py` waives only the decision-record rule for a flip (its rules are now one
  `failures` function, so the test shows the other two still fire). `gh_client` gains
  `pull_files`. `SWITCH` and `LOCK` moved to `merge_flip`, and `merge_train` imports them.
- `scripts/test_merge_flip.py`, 9 tests: the one-line flip passes, and these are refused: an extra
  file or line, `train`/`shadow`, a duplicate key, a nested `mode`, a non-string mode, a symlink, a
  type change, an executable bit, a rename, a non-UTF-8 blob, a files-API listing with two files,
  a rename or another blob, and a head that moved during the check.
- routines.md has the merger row and prompt (`make merge-train TRAIN=1`/`SHADOW=1`, since the
  Makefile requires the flag) and the mode-aware worker and steward merge clauses. In `lock`
  mode they ask for exactly today's flow. SPEC-0027's draft blocks now point to routines.md, and
  docs/status.md lists the owner steps under "Needs owner".
- Eval delta: none (scripts and docs only).
