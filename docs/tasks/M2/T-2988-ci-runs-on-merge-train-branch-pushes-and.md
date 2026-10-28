---
id: T-2988
title: "CI runs on merge-train branch pushes and classifies docs-only trains"
milestone: M2
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: [T-2984]
spec: docs/specs/SPEC-0027-merge-train.md
---

## Goal
- CI runs on pushes to `merge-train/current` and `merge-train/shadow`, and classifies a train's
  changes against its base on `main` the way it does a PR's.
- A docs-only PR whose branch is behind `main` is classified docs-only.
- The full eval report runs only on `main`, or when a gate fails.

## Context
- SPEC-0027 section 9 is the design:
  - the `push` trigger gains exactly `merge-train/current` and `merge-train/shadow`.
    `merge-train/revert-*` and `merge-train/probe` must not trigger it; a revert PR gets
    `pull_request` CI;
  - the `changes` job is keyed on `github.ref`:
    - for those two refs (a push or a `workflow_dispatch`), the base is
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
  - run `git merge-base --all "$BASE" "$PR_HEAD"`, with
    `PR_HEAD = github.event.pull_request.head.sha`, and require **exactly one** base. With
    several bases, none, or a failed command, set `code=true`;
  - diff that single base to `"$PR_HEAD"`, so that only the PR's own changes count;
  - a docs-only result only skips jobs on the PR's own CI. When the PR merges through the train,
    the train's CI classifies the combined diff, so the train stays the gate;
  - first confirm the mechanism (for example, that `base.sha` lags the first parent of the test
    merge being checked out) and record it in the PR.
- The spec rejects the alternative of a draft PR per train.

## Acceptance criteria
- [x] `.github/workflows/ci.yml` makes the changes above. Pushes to `main`, `workflow_dispatch` on
      `main`, and merge groups behave as before. `merge-lock` still triggers nothing.
- [x] `ci_changes` unit cases cover:
      - the train base;
      - a failed merge-base (`code=true`);
      - a PR with several merge bases (`code=true`);
      - **a docs-only PR branch that is behind `main`**, which is classified docs-only.
- [x] The trigger is tested: pushes to `merge-train/revert-x` and `merge-train/probe` start no CI
      run (in the live check, or by a test that parses the workflow's `on.push.branches`).
- [x] The secrets and permissions check has a test, and runs in `make hygiene`.
- [x] A live check, made **while holding the merge lock** so that no merge races it, with the run
      links in the PR (run right after the merge; see Outcome):
      - a docs-only train pushed to `merge-train/current` runs only `changes`, `hygiene` and the
        `rust (fmt, clippy, test)` summary, and is green;
      - a code train runs every job;
      - neither runs the full eval report;
      - a `workflow_dispatch` on the branch is classified like the push;
      - a docs-only PR that is behind `main` gets `run_heavy=false`.
- [x] A decision record lands in this PR. The architecture and security reviewers approve.

## Out of scope
- The train builder (T-2987, T-2994).

## Outcome
- `ci.yml` runs on pushes to exactly `main`, `merge-train/current` and `merge-train/shadow`.
  `changes` is keyed on `github.ref`: a train is diffed from `git merge-base origin/main HEAD`
  with the classifier read from that base; a PR is diffed from the single merge base of
  `base.sha` and its head (`ci_changes.py --merge-base`); every doubtful case gives `code=true`.
  The eval full report runs only on `main` or after a failure. Decision 0147.
- The bug, confirmed on #2119's run 40622986699: `base.sha` (3e0bd12) lagged the test merge's
  first parent (f778a3a), so the diff picked up #2118's crate files and a docs-only head ran the
  heavy jobs. The merge-base diff of the same head is docs-only.
- New hygiene check `scripts/check_ci_workflow.py` (no secrets or token references; every
  `permissions` block exactly `contents: read`). 32 tests across `test_ci_changes.py` and
  `test_check_ci_workflow.py`; the trigger negatives (`merge-train/revert-*`, `probe`,
  `merge-lock`) are pinned by the test that parses `on.push.branches`.
- Live check: the classifier is read from the base on `main`, so a docs-only train or PR can
  only classify docs-only once this is on `main`. It is run right after the merge, under the
  merge lock: a docs-only train, a `workflow_dispatch` on it, a code train, and a docs-only PR
  behind `main`. Run links are posted on the PR.
- No eval delta (no engine change).
