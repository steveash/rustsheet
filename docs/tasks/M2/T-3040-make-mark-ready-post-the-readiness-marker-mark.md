---
id: T-3040
title: "make mark-ready: post the readiness marker (mark, --carry, labels, step-mode fallback)"
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
A worker marks a green, approved PR ready for the merge train with one command, and the marker
it posts passes `scripts/merge_ready.py verify`.

## Context
- Split from T-2986, which built the marker format, reviewer applicability, the "green" rule and
  `verify` (V1-V6) in `scripts/merge_ready.py` (decision 0149). Reuse them; do not copy them.
- SPEC-0027 section 2 is the design: who may mark, the refusals, `--carry` (a clean merge of
  `main`: first parent `prev`, second parent on `origin/main`, `git merge-tree --write-tree`
  reproduces the head's tree), and "never a second marker for the same head".
- T-3001 found that `gh api` writes do not run in a routine session, so `mark` needs the
  minimal write fallback: exactly two step-mode JSON actions (the label and a comment whose body
  the script generates), in SPEC-0027 section 4's schema ("GitHub access"). T-2995 extends it.
  Adding a label through the REST issue-labels endpoint creates a missing label.

## Acceptance criteria
- [x] `scripts/merge_ready.py mark [--carry]` (`make mark-ready PR=<n> REVIEW="..."`) refuses
      each case in SPEC-0027 section 2, posts the marker and the label otherwise (or prints the
      two step-mode actions), and never posts a second marker for the same head.
- [x] The labels `ready-to-merge`, `merge-train-ejected` and `merge-train-stop` exist; the
      script creates them when they are absent (through `scripts/gh_client.py`).
- [x] `verify` passes on what `mark` posts, and a test covers the step-mode round trip.
- [x] `scripts/test_merge_ready.py` covers the `--carry` cases of SPEC-0027's Test plan (a clean
      merge of `main` is accepted, an edited one refused) and each refusal.
- [x] A decision record amends 0149; the architecture and security reviewers approve.
- [x] `make check` green; reviewers approve.

## Out of scope
- Skill and prompt changes (T-2989).

## Outcome
- `scripts/merge_ready.py mark` (`make mark-ready PR=<n> REVIEW="..." [CARRY=1] [STEP=1]`) refuses
  each case of SPEC-0027 section 2, never posts a second marker for a head, writes the comment
  before the label, and either writes through `gh_client` and reads the result back with
  `verify`, or (`--step`) prints the two writes as step-mode JSON actions (decision 0161).
- `--carry` takes the verdicts and `prev:` from the newest earlier writer marker, only for a clean
  merge of `main` (`Repo.carry_problem`); `verify` and `mark` share `head_state`.
- `gh_client` gains `label` and `create_label`; `make merge-labels` created `ready-to-merge`,
  `merge-train-ejected` and `merge-train-stop` on the repository (2026-10-29).
- Tests: the step-mode round trip passes `verify`, direct writes in order, each refusal, the
  `--carry` cases (clean merge accepted; edited merge, plain commit and a non-`main` merge refused).
- Eval delta: none (scripts only).
