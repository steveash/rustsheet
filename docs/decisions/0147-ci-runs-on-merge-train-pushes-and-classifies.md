# 0147 — CI runs on merge-train pushes and classifies a PR against its merge base

- **Status:** accepted (2026-10-28, T-2988). It amends 0049 and 0066 and implements SPEC-0027
  section 9 ("CI on train branches"). Nothing pushes to the train branches until `mode` says so
  (SPEC-0027, Rollout), so until then only the PR classification change is in force.
- **Context:**
  - The merge train (SPEC-0027) tests the exact sha it builds on `merge-train/current` or
    `merge-train/shadow`, so CI must run on pushes to those branches. Today only `main` triggers
    on `push`.
  - The `changes` job diffed `github.event.pull_request.base.sha` against `HEAD`, GitHub's test
    merge. `base.sha` can lag the test merge's first parent: in run 40622986699 (PR #2119, head
    a622e22, which touched only `docs/`) `BASE` was 3e0bd12 (#2114) while the checkout was
    "Merge a622e22 into f778a3a" (#2118 merged in between). The diff therefore also listed
    #2118's crate files and the job set `code=true` ("changed files: 36"). Decision 0121 leaves
    branches behind `main` until they hold the lock, so docs-only PRs often paid for the full CI.
    The single merge base of 3e0bd12 and a622e22 is d28666e, and diffing it to a622e22 is
    docs-only.
- **Decision:**
  - `on.push.branches` is exactly `main`, `merge-train/current` and `merge-train/shadow`
    (literal names, no globs), so `merge-train/revert-*`, `merge-train/probe` and `merge-lock`
    trigger nothing; a revert PR gets `pull_request` CI.
  - The `changes` job is keyed on `github.ref`. A pull request diffs from the single merge base
    of `base.sha` and `head.sha` to `head.sha`, so only the PR's own changes count; the
    classifier is still read from `base.sha` (0066). The two train refs (a push or a
    `workflow_dispatch`) diff from the merge base of `origin/main` and `HEAD`, reading the
    classifier from that base. Every other ref (`main`, merge groups, dispatches elsewhere) runs
    everything.
  - The merge-base step lives in `scripts/ci_changes.py --merge-base A B`: it requires exactly
    one merge base (`git merge-base --all`) and a complete (not shallow) clone; several bases,
    none, a failed git command, a shallow clone or a malformed revision give `code=true`.
  - The eval "Full report" step runs on `failure() || github.ref == 'refs/heads/main'`: trains
    and dispatches on other branches skip it unless a gate fails (0054). `governance` stays
    PR-only and the `ci-${{ github.ref }}` concurrency group is unchanged.
  - `scripts/check_ci_workflow.py` (in `make hygiene`) fails when `ci.yml` references
    `secrets.`, `secrets:` or `github.token`, or when any `permissions` block is anything but
    `contents: read`. Train CI runs with the same rights as PR CI.
- **Consequences:**
  - A docs-only PR behind `main` skips the heavy jobs on its own CI. When it merges through the
    train, the train's CI classifies the combined diff, so the train stays the gate; under the
    current serial merge the PR's under-lock run (after merging `main`) still runs on its own
    merge base and is classified the same way.
  - The `--merge-base` mode takes effect for PRs whose base already holds it (0066): an older
    base copy ignores the arguments, reads empty stdin and runs everything.
  - Changing the trigger list or the rights `ci.yml` holds now needs the check (and this record)
    changed too.
