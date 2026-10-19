# 0053 — Merge through the GitHub merge queue instead of the up-to-date check

- **Status:** accepted (2026-10-13, owner request, T-1505). It amends 0028: once the queue is on,
  the queue gives 0028's guarantee and the agents' mechanical up-to-date check is skipped. Dormant per 0075: a
  repository owned by a personal account cannot turn the queue on, so merges go through the merge lock
  until the repository moves to an organization and the queue is on.
- **Context:** Decision 0028 requires a PR head to contain the current `main` before it merges,
  and the ruleset enforces "require branches to be up to date". With about 2.7 merges an hour and
  a 13-minute CI (the python job), a green PR is usually stale again before it can merge. On
  2026-10-12, #586 needed five CI rounds, and #611 and #624 two or three each. Every round is a
  full CI run plus an agent wake-up.
- **Decision:**
  - CI runs on the `merge_group` event too, running every job (the docs-only shortcut of decision
    0049 only applies to `pull_request`). The `governance` step stays PR-only: it needs the PR's
    base ref, and it has already passed on the PR head the queue merges. The queue tests each PR merged with current `main` and
    the PRs queued ahead of it, so `tasks.py validate`, `check_decisions.py`, the ratchets and the
    eval gate still see the real merged tree: duplicate task or decision ids still fail before
    they land (0028's reason).
  - The owner turns the queue on in the `main` ruleset (settings below). Until then, nothing
    changes: agents merge per 0028.
  - Once it is on, agents enqueue instead of merging: on a green, reviewed head, call
    `enable_pr_auto_merge` with `mergeMethod: SQUASH`. The merge-queue signals are a merge call
    refused with "merge queue" in the message, or a PR whose state shows it queued. Agents never
    merge `main` in just because the branch is behind; they still merge it in to resolve a
    conflict. If the queue removes a PR (a check failed on the merge group), that is red CI on
    that PR, handled per the `steward` skill.
  - A queued PR gets no new pushes: auto-merge stays armed across a push, so new commits could
    land unreviewed. To change a queued PR, first `disable_pr_auto_merge`, push, re-run the
    reviewers on the new diff, then enqueue again.
  - Owner settings (Settings → Rules → the ruleset on `main`): turn on "Require merge queue"
    (merge method Squash, build concurrency 5, minimum group size 1, maximum group size 5,
    "only merge non-failing pull requests", status check timeout 60 minutes), and keep the
    existing required checks unchanged: `changes (docs-only?)`, `rust (fmt, clippy, test)`,
    `eval (Excel-compatibility gate)`, `python (tools + bindings)` and `hygiene (ratchets, deny,
    machete, typos, governance)`. Turn off
    "Require branches to be up to date before merging" (the queue replaces it), and turn on
    "Allow auto-merge" under Settings → General.
- **Consequences:** a PR costs one CI run on its own head and one on its merge group. A busy
  `main` no longer means repeated update rounds, and groups of up to five PRs share one run. A
  failing group drops the culprit and retests the rest. Concurrency: merge-group refs are unique
  per group, so the existing `ci-${{ github.ref }}` group does not cancel them.
