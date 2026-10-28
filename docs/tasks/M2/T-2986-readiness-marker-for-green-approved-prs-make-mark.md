---
id: T-2986
title: "Readiness marker for green, approved PRs: make mark-ready and its verifier"
milestone: M2
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2984]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
A worker can mark a green, approved PR ready for the merge train with one command. Anyone can
then check that marker mechanically, without reviewing the PR again.

## Context
- SPEC-0027 section 2 ("The readiness handoff") is the design:
  - the `ready-to-merge` label;
  - the `merge-train-ready v1` marker comment (head sha, task, reviewer verdicts, marked-by);
  - who may mark;
  - how a push invalidates the marker;
  - verification checks V1–V6.
- One definition (CLAUDE.md directive 3): the marker parser and the reviewer-applicability rule
  live in `scripts/merge_ready.py`, and the train builder (T-2987) imports them.
- GitHub access goes through a small client interface. Production uses `gh api`: reads through
  the session proxy were verified on 2026-10-28. The tests use a fake.

## Acceptance criteria
- [ ] `scripts/merge_ready.py mark` (`make mark-ready PR=<n> REVIEW="..."`) refuses each case
      listed in SPEC-0027 section 2. Otherwise it posts the marker and adds the label. It never
      posts a second marker for the same head.
- [ ] `scripts/merge_ready.py verify --pr <n>` implements V1–V6. It exits 0 when the PR is ready,
      and 1 with the failed checks when it is not.
- [ ] The labels `ready-to-merge` and `merge-train-ejected` exist. The script creates them when
      they are absent.
- [ ] `scripts/test_merge_ready.py` covers every `test_merge_ready.py` case in SPEC-0027's Test
      plan, and runs in `make hygiene`.
- [ ] Nothing in the skills or routines uses the script yet (T-2989 does that).
- [ ] A decision record for the mechanism lands in this PR. The architecture and security
      reviewers approve.

## Out of scope
- Building or merging trains (T-2987).
- Skill and prompt changes (T-2989).
