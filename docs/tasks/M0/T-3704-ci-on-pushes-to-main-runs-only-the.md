---
id: T-3704
title: "CI on pushes to main runs only the eval report"
milestone: M0
lane: harness
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A push to main runs only what main needs (the informational full eval report and its artifact),
not a second copy of the gates the merge train already ran on the exact merged sha.

## Context
Owner request (2026-11-01, CI-speed investigation). On 2026-11-01 (05:53-09:16Z) 12 of 16 CI runs
on main were cancelled by the next merge's run (concurrency group `ci-${{ github.ref }}`), after
0.2-5.6 min each, so the per-merge eval report of decision 0054 mostly does not exist. In `train`
mode (scripts/merge_train.toml) the train branch's CI tests the sha that lands.

## Acceptance criteria
- [x] A decision record states when main's push run may be trimmed (only when the sha was tested on
      `merge-train/current`, or always) and what keeps the nightly/full eval coverage; it amends
      0054 and 0147.
- [x] CI on main runs the report job without being cancelled by the next merge (e.g. no
      cancel-in-progress for main, or a separate workflow), and skips the duplicated gates if the
      decision allows.
- [x] `scripts/check_ci_workflow.py` and its tests agree; `make check` green; reviewers approve.

## Out of scope
- PR or merge-train CI.

## Blocked because
Blocked because: needs an owner ruling. The change contradicts a decision SPEC-0027 made on
purpose.

What #2739 does: it sets `cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}`, with
decision 0239 drafted on the branch `agent/harness/T-3704-main-ci-not-cancelled`. The quality
reviewer requested changes on 50e43ffc6.

What SPEC-0027 says:
- Section 5, "Intermediate states", decides that `main`'s push runs are cancelled on purpose. The
  train already tested `S_n`'s tree, and the intermediate squashes only confirm prefixes of it.
  Back-to-back trains may also cancel `S_n`'s own run.
- The daily steward run keeps one full report a day.
- Rejected alternative 17 lists exempting `main` from `cancel-in-progress`.

The 12 of 16 cancelled `main` runs measured on 2026-11-01 are that design working, not a defect.

The spec's cost argument ("multiplies `main`'s CI cost by n") overstates the cost. GitHub keeps at
most one pending run per concurrency group, so with no cancelling on `main` a train of n squashes
runs `S_1`, the first prefix, and `S_n`, about one extra run per train. The gain is small: a
finished run and report on `S_n` even when the next train lands at once, and a test of `S_1`.

Options for the owner:
- (a) drop the task, keeping SPEC-0027 as it is;
- (b) take #2739 after amending SPEC-0027 section 5 and alternative 17 and citing 0147, as the
  quality reviewer asked;
- (c) keep cancelling on `main`, but let the steward's daily full eval report (already the
  spec's backstop) record how many squashes went untested that day.

