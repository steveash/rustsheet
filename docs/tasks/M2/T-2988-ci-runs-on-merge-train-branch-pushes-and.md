---
id: T-2988
title: "CI runs on merge-train branch pushes and classifies docs-only trains"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-2984]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
CI runs on pushes to `merge-train/**` and classifies a train's changes against the train's base
on `main`, as it does for a PR. The full eval report runs only on `main`.

## Context
- SPEC-0027 section 9 is the design:
  - the `push` trigger gains `'merge-train/**'`;
  - for a train push, the `changes` job takes `git merge-base origin/main HEAD` as the base and
    runs that base's `scripts/ci_changes.py` (decision 0066);
  - the eval job's "Full report" condition becomes `github.ref == 'refs/heads/main'`;
  - `governance` stays PR-only;
  - the `ci-${{ github.ref }}` concurrency group cancels superseded trains.
- The spec rejects the alternative of a draft PR per train.

## Acceptance criteria
- [ ] `.github/workflows/ci.yml` makes the changes above. Pushes to `main`, `workflow_dispatch`
      and merge groups behave as before, and `merge-lock` still triggers nothing.
- [ ] A live check from a session, with the run links in the PR:
      - a docs-only train pushed to `merge-train/current` runs only `changes`, `hygiene` and the
        `rust (fmt, clippy, test)` summary, and is green;
      - a code train runs every job;
      - neither runs the full eval report.
- [ ] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- The train builder (T-2987).
