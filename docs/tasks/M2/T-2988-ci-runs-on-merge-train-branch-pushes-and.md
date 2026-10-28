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
CI runs on pushes to `merge-train/**` and classifies a train's changes against its base on `main`,
as it does for a PR. The full eval report runs only on `main`, or when a gate fails.

## Context
- SPEC-0027 section 9 is the design:
  - the `push` trigger gains `'merge-train/**'`;
  - the `changes` job is keyed on `github.ref`. For `refs/heads/merge-train/*` (a push or a
    `workflow_dispatch`) the base is `git merge-base origin/main HEAD`, and `scripts/ci_changes.py`
    is read from that base (decision 0066). If the merge-base fails, everything runs. Pull
    requests are unchanged, and every other ref runs everything;
  - the eval job's "Full report" condition becomes `failure() || github.ref == 'refs/heads/main'`,
    so dispatches on other branches deliberately skip it;
  - `governance` stays PR-only;
  - the `ci-${{ github.ref }}` concurrency group cancels superseded trains;
  - a hygiene check fails if `ci.yml` references `secrets.` or widens `permissions` beyond
    `contents: read`.
- The spec rejects the alternative of a draft PR per train.

## Acceptance criteria
- [ ] `.github/workflows/ci.yml` makes the changes above. Pushes to `main`, `workflow_dispatch` on
      `main`, and merge groups behave as before. `merge-lock` still triggers nothing.
- [ ] `ci_changes` unit cases cover the train base, including a failed merge-base, which must
      produce `code=true`. The secrets and permissions check has a test, and runs in
      `make hygiene`.
- [ ] A live check, made **while holding the merge lock** so that no merge races it, with the run
      links in the PR:
      - a docs-only train pushed to `merge-train/current` runs only `changes`, `hygiene` and the
        `rust (fmt, clippy, test)` summary, and is green;
      - a code train runs every job;
      - neither runs the full eval report;
      - a `workflow_dispatch` on the branch is classified like the push.
- [ ] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- The train builder (T-2987, T-2994).
