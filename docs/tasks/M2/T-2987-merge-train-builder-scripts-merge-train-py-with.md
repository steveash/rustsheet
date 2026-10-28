---
id: T-2987
title: "Merge-train builder scripts/merge_train.py with a dry-run mode"
milestone: M2
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2985, T-2986, T-2988]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
`make merge-train` runs one batch of SPEC-0027's algorithm from start to finish under the fair
merge lock. A dry-run mode writes nothing, and a shadow mode tests trains but merges nothing. A
switch picks the mode, and it defaults to off.

## Context
- SPEC-0027 sections 4–7 are the design:
  - collect ready PRs, FIFO by marker time, `BATCH_MAX` 5;
  - build `merge-train/current` from `origin/main` with merge commits and the independence check;
  - run the id checks with `M0`'s copies of `tasks.py validate` and `check_decisions.py`, adding
    the `--root` option they need;
  - push, and wait for CI on the built sha;
  - squash-merge in order with `expectedHeadSha` and a tree check after each squash;
  - bisect by prefix and eject the culprit;
  - recover from crashes.
- The switch is `scripts/merge_train.toml` (`mode = "lock" | "shadow" | "train"`), read by
  `make merge-mode`. This task ships it set to `lock`.
- SPEC-0027 open question 2: confirm that `gh api` writes (merge, label, comment) work from a
  session. If they do not, add the JSON step mode the spec describes, so that the session makes
  each write with the GitHub MCP tools.

## Acceptance criteria
- [ ] `scripts/merge_train.py` implements SPEC-0027 sections 4–7. `make merge-train [DRY_RUN=1]
      [SHADOW=1]` and `make merge-mode` exist, and `merge_train.toml` says `mode = "lock"`.
- [ ] `scripts/test_merge_train.py` covers every `test_merge_train.py` case in SPEC-0027's Test
      plan, including the tree-equality property test, and runs in `make hygiene`.
- [ ] A dry run against the live repository prints a plausible batch and writes nothing. The
      output is pasted in the PR.
- [ ] The answer to open question 2 is recorded in the PR and in the spec.
- [ ] A decision record for the mechanism lands in this PR. The architecture and security
      reviewers approve.

## Out of scope
- Skill, process and routine changes (T-2989).
- Any mode other than `lock` (T-2989, T-2991).
