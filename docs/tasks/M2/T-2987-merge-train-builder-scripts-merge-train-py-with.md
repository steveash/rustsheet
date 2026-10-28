---
id: T-2987
title: "Merge-train builder scripts/merge_train.py with a dry-run mode"
milestone: M2
lane: harness
status: done
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
- [x] `scripts/merge_train.py --dry-run` implements steps 2–4 and makes no writes (no push, no
      label, no comment, no lock).
- [x] `tasks.py validate --root` and `check_decisions.py --root` exist and are tested.
- [x] `scripts/test_merge_train.py` covers SPEC-0027's Test plan cases for collect, build,
      independence (criss-cross included), the id checks and dry-run. It includes the
      tree-equality property test, and runs in `make hygiene`.
- [x] `scripts/merge_train.toml` says `mode = "lock"`, and `make merge-mode` reads it from
      `origin/main` and fails closed. A test covers a missing file, a malformed file, a duplicate
      key and an unknown value.
- [x] A test shows that a planted `tasks.py` or module in the train tree is not imported by the id
      checks.
- [x] A dry run against the live repository prints a plausible batch; the output is pasted in the
      PR.
- [x] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- Push, CI, merging and the lock (T-2994). Bisection (T-2998).
- Shadow mode and the step-mode fallback (T-2995).

## Outcome
- `scripts/merge_train.py`: `mode` (`make merge-mode`) reads `scripts/merge_train.toml` from a
  fresh `origin/main` and fails closed; `--dry-run` (`make merge-train DRY_RUN=1`) collects with
  `merge_ready.verify`, builds up to 5 PRs in scratch worktrees (independence by
  `merge-base --all`, deferral, ejection), runs `main`'s id checks after each merge with
  `python3 -I`, and prints the batch. It writes nothing.
- `tasks.py validate --root` and `check_decisions.py --root`; `scripts/merge_train.toml` says
  `mode = "lock"`. Decision 0150.
- `scripts/test_merge_train.py`: the fail-closed mode read, the id checks and a planted module,
  build (cap, defer, eject, criss-cross, id collisions), the tree-equality property test (with
  renames, deletions and merges of `main` into PRs), and a dry run that writes nothing.
- Live (2026-10-28, `main` 45bacab): `make merge-train DRY_RUN=1` printed "0 candidate(s)": no
  PR carries the label yet (`mark` is T-3040). Building the first 5 open non-draft PRs directly
  (#2191, #2190, #2189, #2162, #2151) merged all 5 with no deferral or ejection.
- Eval delta: none (scripts and docs only).
