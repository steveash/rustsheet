---
id: T-3455
title: "Merge tooling unshallows its clone so docs-only PRs are not ejected as CI red"
milestone: M0
lane: harness
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
A docs-only PR (task filings, gap triage, plans) that is green and approved stays `ready-to-merge`
and merges through the train, even when the merger runs in a shallow session clone.

## Context
Since the train was switched on (#2547, 2026-10-31 10:20 UTC) no docs-only PR has merged. Each one
is unlabelled with `CI red on <head>: eval (Excel-compatibility gate): skipped; python (tools +
bindings): skipped` (#2574 at 13:40, #2577 at 13:54). CI's own classifier ran on a full clone and
rightly skipped the heavy jobs. The merger's V3 check (`merge_ready.head_state`) then asks
`ci_changes.classify_range` whether the diff is docs-only, so that skipped heavy jobs count as
green. In a shallow clone, which is how cloud sessions clone the repo (`git rev-parse
--is-shallow-repository` prints `true`, about 50 commits deep), `classify_range` answers "code" on
purpose (it cannot trust the merge base). So every skipped heavy job reads as red. The train's own
check of its tip (`merge_train.py`, `classify_range(train.base, tip)`) is affected in the same way.

Reproduced 2026-10-31 in a session clone: `classify_range(main, #2574 head)` and `(main, #2577
head)` give `True` (code) while the clone is shallow. After `git fetch --unshallow origin main`
(3 s, about 6 MB) both give `False` (docs-only).

## Acceptance criteria
- [x] `merge_ready.Repo.fetch_base` turns a shallow clone into a full one before it fetches `main`.
  The V3 check in `verify`, `mark`, the train (whose `collect` reads through the same `Repo` and
  root) and merge-train-stats then classify with full history. A full clone is left as it is.
- [x] A test builds a shallow clone of a fixture origin. It shows that a docs-only diff with skipped
  heavy jobs is GREEN after `fetch_base`, and that the clone is no longer shallow.
- [x] `ci_changes.classify_range` keeps answering "code" in a shallow clone (CI's classifier is
  unchanged).
- [x] Decision record for the protected `scripts/` change.
- [x] `make check` green; reviewers approve.

## Out of scope
- Re-marking the PRs the bug already ejected. Their owners re-run `make mark-ready` once this lands.

## Outcome
Done. `merge_ready.Repo.fetch_base` now adds `--unshallow` to its fetch of `main` when the clone
is shallow (decision 0201). The new test `test_a_shallow_clone_is_deepened_before_the_docs_only_check`
clones the fixture origin `--depth 1`. Before the fix, the docs-only PR with skipped heavy jobs
failed V3 (unlabel). Now it is ready, and the clone ends up non-shallow. A second `verify` on the
full clone also passes, because `--unshallow` is not repeated. The 102 tests in
`test_merge_ready`, `test_merge_train*` and `test_merge_step` pass. In a session clone,
`classify_range` on #2574 and #2577 gave code while the clone was shallow and docs-only after
unshallowing it. The docs-only PRs ejected since 10:20 UTC (#2574, #2577 and the other task-only
PRs without the label) need `make mark-ready` again.
