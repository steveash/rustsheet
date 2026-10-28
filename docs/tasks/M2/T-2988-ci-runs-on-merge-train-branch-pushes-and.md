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
- CI runs on pushes to `merge-train/**`, and classifies a train's changes against its base on
  `main` the way it does a PR's.
- A docs-only PR whose branch is behind `main` is classified docs-only.
- The full eval report runs only on `main`, or when a gate fails.

## Context
- SPEC-0027 section 9 is the design:
  - the `push` trigger gains `'merge-train/**'`;
  - the `changes` job is keyed on `github.ref`:
    - for `refs/heads/merge-train/*` (a push or a `workflow_dispatch`), the base is
      `git merge-base origin/main HEAD`, and `scripts/ci_changes.py` is read from that base
      (decision 0066). If the merge-base fails, everything runs;
    - every other non-PR ref runs everything;
  - the eval job's "Full report" condition becomes `failure() || github.ref == 'refs/heads/main'`,
    so dispatches on other branches deliberately skip it;
  - `governance` stays PR-only;
  - the `ci-${{ github.ref }}` concurrency group cancels superseded trains;
  - a hygiene check fails if `ci.yml` references `secrets.` or widens `permissions` beyond
    `contents: read`.
- **The PR classification bug** (SPEC-0027 section 9). Today the job diffs `"$BASE" HEAD` with
  `BASE = github.event.pull_request.base.sha`. A docs-only PR whose branch is behind `main` was
  classified `code=true`: on #2119, a622e22 touched only `docs/` and ran with `run_heavy=true`.
  Decision 0121 leaves branches behind until they hold the lock, so docs-only PRs often pay for
  the full CI. The fix:
  - diff `git merge-base "$BASE" "$PR_HEAD"` to `"$PR_HEAD"`, with
    `PR_HEAD = github.event.pull_request.head.sha`, so that only the PR's own changes count;
  - fall back to `code=true` if the merge-base fails;
  - first confirm the mechanism (for example, that `base.sha` lags the first parent of the test
    merge being checked out) and record it in the PR.
- The spec rejects the alternative of a draft PR per train.

## Acceptance criteria
- [ ] `.github/workflows/ci.yml` makes the changes above. Pushes to `main`, `workflow_dispatch` on
      `main`, and merge groups behave as before. `merge-lock` still triggers nothing.
- [ ] `ci_changes` unit cases cover:
      - the train base;
      - a failed merge-base (`code=true`);
      - **a docs-only PR branch that is behind `main`**, which is classified docs-only.
- [ ] The secrets and permissions check has a test, and runs in `make hygiene`.
- [ ] A live check, made **while holding the merge lock** so that no merge races it, with the run
      links in the PR:
      - a docs-only train pushed to `merge-train/current` runs only `changes`, `hygiene` and the
        `rust (fmt, clippy, test)` summary, and is green;
      - a code train runs every job;
      - neither runs the full eval report;
      - a `workflow_dispatch` on the branch is classified like the push;
      - a docs-only PR that is behind `main` gets `run_heavy=false`.
- [ ] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- The train builder (T-2987, T-2994).
