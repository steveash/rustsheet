---
id: T-2987
title: "Merge-train builder scripts/merge_train.py with a dry-run mode"
milestone: M2
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2986]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
`make merge-train DRY_RUN=1` collects the ready PRs, builds the train locally and runs the id
checks, then prints the batch, the deferrals and the ejections. It writes nothing. The switch
file exists and says `lock`.

## Context
- SPEC-0027 section 4, steps 2–4, and "Dry run":
  - collect: V1–V6 come from `merge_ready.py`; FIFO by marker `updated_at`; `BATCH_MAX` 5;
  - build from `origin/main` with `git merge --no-ff`, fetching `refs/pull/<n>/head` and comparing
    it with `head.sha` and the marker;
  - the independence check uses `git merge-base --all`: every base must be an ancestor of `M0`;
  - defer or eject on conflicts;
  - id checks with `M0`'s copies of `tasks.py validate` and `check_decisions.py`, through a new
    `--root <dir>` option on both. They run as `python3 -I <M0 scripts worktree>/scripts/...
    --root <train worktree>`, with the working directory outside the train tree, so that no
    module from the train is imported;
  - ejection of the later colliding PR.
- SPEC-0027 section 5 is the argument the property test checks.
- The switch: `scripts/merge_train.toml` (`mode = "lock"`) and `make merge-mode`, which reads it
  from `origin/main` after a fetch, never from the checkout. The read fails closed: a missing file,
  a parse error, a duplicate key, or a missing or unknown `mode` reads as `lock`.
- The write policy (SPEC-0027 section 4): a dry run makes no write of any kind, and it prints the
  would-be unlabels and ejections.

## Acceptance criteria
- [ ] `scripts/merge_train.py --dry-run` implements steps 2–4 and makes no writes (no push, no
      label, no comment, no lock).
- [ ] `tasks.py validate --root` and `check_decisions.py --root` exist and are tested.
- [ ] `scripts/test_merge_train.py` covers SPEC-0027's Test plan cases for collect, build,
      independence (criss-cross included), the id checks and dry-run. It includes the
      tree-equality property test, and runs in `make hygiene`.
- [ ] `scripts/merge_train.toml` says `mode = "lock"`, and `make merge-mode` reads it from
      `origin/main` and fails closed. A test covers a missing file, a malformed file, a duplicate
      key and an unknown value.
- [ ] A test shows that a planted `tasks.py` or module in the train tree is not imported by the id
      checks.
- [ ] A dry run against the live repository prints a plausible batch; the output is pasted in the
      PR.
- [ ] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- Push, CI, merging and the lock (T-2994). Bisection (T-2998).
- Shadow mode and the step-mode fallback (T-2995).
