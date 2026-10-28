---
id: T-2986
title: "Readiness marker for green, approved PRs: make mark-ready and its verifier"
milestone: M2
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-3001]
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
    T-3001 found the permission API refused by the session proxy (HTTP 403; SPEC-0027 section 4,
    "T-3001's probe"): in a session V1 and V6 need another source, and `author_association` is
    the candidate there.
- SPEC-0027 section 4, step 6, defines "green". It lives here as the single definition, used for
  V3 and by the train builder. It requires success for `changes`, `hygiene` and the rust summary;
  allows `skipped` only when `run_heavy=false`; never counts a cancelled run as green; and counts
  only check runs from the GitHub Actions app.
- V6 counts only reviews whose author passes V1's permission lookup. Reviews by anyone else are
  ignored with no side effects. V2–V5 read only repository-controlled data (SPEC-0027 section 2).
- Reviewer applicability is defined in one place, the `review` skill
  (`.claude/skills/review/SKILL.md`, step 3). `merge_ready.py` holds the same list, and a test
  fails when the two differ. That test reads `.claude/`, so this task adds the file to
  `CODE_READ_PATHS` in `scripts/ci_changes.py` (decision 0066).
- It uses `scripts/gh_client.py` (T-3001) for every GitHub call. If T-3001's probe found that
  writes fail, `mark` uses its minimal fallback: it prints its two writes (the label, and a
  comment whose body the script generates) as step-mode JSON actions, in the schema of SPEC-0027
  section 4 "GitHub access". T-2995's step mode extends the same schema.

## Acceptance criteria
- [ ] `scripts/merge_ready.py mark [--carry]` (`make mark-ready PR=<n> REVIEW="..."`) refuses
      each case in SPEC-0027 section 2, posts the marker and the label otherwise, and never posts
      a second marker for the same head.
- [ ] `scripts/merge_ready.py verify --pr <n>` implements V1–V6 and the "green" rule. It exits 0
      when the PR is ready and 1 with the failed checks when it is not.
- [ ] The labels `ready-to-merge`, `merge-train-ejected` and `merge-train-stop` exist; the script
      creates them when they are absent.
- [ ] If T-3001 found that `gh api` writes fail, `mark` uses the minimal write fallback. It prints
      exactly two step-mode JSON actions, the label and a comment whose body the script generates,
      in SPEC-0027 section 4's schema. `verify` confirms the result afterwards, and a test covers
      it. If writes work, the PR says the fallback was not needed and omits it.
- [ ] `scripts/test_merge_ready.py` covers every `test_merge_ready.py` case in SPEC-0027's Test
      plan, including an outsider's `CHANGES_REQUESTED` review and the test that the
      applicability list matches the review skill. It runs in `make hygiene`.
- [ ] No skill or routine uses it yet (T-2989 does that).
- [ ] A decision record for the mechanism lands in this PR. The architecture and security
      reviewers approve.
- [ ] If the diff grows past size M, split it before review: the parser and `verify` first, then
      `mark`.

## Out of scope
- Building or merging trains (T-2987, T-2994).
- Skill and prompt changes (T-2989, T-2996).
- Per-reviewer verdict records (T-2997).
