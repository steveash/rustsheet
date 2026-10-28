---
id: T-2986
title: "Readiness marker for green, approved PRs: make mark-ready and its verifier"
milestone: M2
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2993]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
A worker marks a green, approved PR ready for the merge train with one command. Anyone can then
check the marker mechanically, without reviewing the PR again.

## Context
- SPEC-0027 section 2 ("The readiness handoff") is the design:
  - the `ready-to-merge` label and the `merge-train-ready v1` marker (head, prev, task, verdicts,
    marked-by);
  - who may mark;
  - `--carry`, a mechanical check that a clean merge of `main` left the PR's diff unchanged;
  - invalidation by a push;
  - trust: the verdicts are an attestation, not a verification;
  - checks V1–V6, each with its failure side effects. V1 keeps only comments whose author has
    write, maintain or admin permission (permission API) and ignores the rest with no side
    effects. The newest qualifying marker by `updated_at` counts, pinned by comment id.
- SPEC-0027 section 4, step 6, defines "green". It lives here as the single definition, used for
  V3 and by the train builder. It requires success for `changes`, `hygiene` and the rust summary;
  allows `skipped` only when `run_heavy=false`; never counts a cancelled run as green; and counts
  only check runs from the GitHub Actions app.
- It uses `scripts/gh_client.py` (T-2993) for every GitHub call. It writes only if T-2993's probe
  showed writes work; otherwise `mark` prints the two actions for the session to perform.

## Acceptance criteria
- [ ] `scripts/merge_ready.py mark [--carry]` (`make mark-ready PR=<n> REVIEW="..."`) refuses
      each case in SPEC-0027 section 2, posts the marker and the label otherwise, and never posts
      a second marker for the same head.
- [ ] `scripts/merge_ready.py verify --pr <n>` implements V1–V6 and the "green" rule. It exits 0
      when the PR is ready and 1 with the failed checks when it is not.
- [ ] The labels `ready-to-merge` and `merge-train-ejected` exist; the script creates them when
      they are absent.
- [ ] `scripts/test_merge_ready.py` covers every `test_merge_ready.py` case in SPEC-0027's Test
      plan, and runs in `make hygiene`.
- [ ] No skill or routine uses it yet (T-2989 does that).
- [ ] A decision record for the mechanism lands in this PR. The architecture and security
      reviewers approve.
- [ ] If the diff grows past size M, split it before review: the parser and `verify` first, then
      `mark`.

## Out of scope
- Building or merging trains (T-2987, T-2994).
- Skill and prompt changes (T-2989, T-2996).
- Per-reviewer verdict records (T-2997).
