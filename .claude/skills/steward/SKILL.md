---
name: steward
description: Repo-specific conventions for driving RustSheet PRs to merge autonomously - CI failure handling, review comments, merge policy. Read before acting on CI or review events on a RustSheet PR.
---
# PR stewardship conventions (RustSheet)

- **Merge policy**: no human approval is required. A PR may be squash-merged by the agent that
  opened it when (a) all required CI jobs are green on the latest head, (b) the `review` skill
  table shows APPROVE from every applicable reviewer, (c) the branch is up to date with `main`.
- **Merge style**: squash merge; title `T-XXXX: <task title>`; keep the Co-Authored-By trailer.
  Exception: the daily steward PR uses branch `agent/steward/<YYYY-MM-DD>` and title
  `steward: <YYYY-MM-DD> health` (docs/process/routines.md).
- **Branch updates**: merge `origin/main` into the branch (never rebase/force-push a pushed branch).
- **Merge conflicts are never "waiting on CI"** (decision 0020). GitHub runs no CI on a PR that
  conflicts with `main`, so no CI event will ever arrive and waiting deadlocks the PR. Check the
  PR's `mergeable_state` (GitHub MCP `pull_request_read` method `get`; if still `unknown` after one
  re-read, test-merge `origin/main` locally, and otherwise re-check at the next check-in):
  - after every push and right after opening the PR;
  - on every PR event or check-in, before anything else;
  - after any PR merges to `main` (yours or another session's): re-check every open PR this
    session opened or drives;
  - whenever a pushed head has no check runs a few minutes later.
  If it is `dirty`: `git fetch origin main`, merge `origin/main` into the branch now, resolve
  (keep both sides' features; status-log conflicts keep both lines; regenerate baselines with
  `make eval-baseline` and take `main`'s side of generated files first), run `make check`, push.
  Do the same for a stale open agent PR (an `agent/*` branch only, never a human's branch) from another session whose task is otherwise done
  (a merge commit only, never a rewrite), and say so on that PR. Ask only when both sides changed
  the same logic and either choice loses behavior.
- **Never idle on an open PR without a wake-up**: after opening a PR, arm a safety-net check-in
  (`send_later`, ~50 min) when the tool exists, so a lost event or an unnoticed conflict is
  caught; re-arm while the PR is open.
- **Generated files**: never hand-edit `eval/baseline/*.txt` or `quality/*.json`; regenerate with
  `make eval-baseline` and the `scripts/check_*.py --update` commands. `docs/tasks/BOARD.md` is
  git-ignored and never committed (decision 0017).
- **CI failures**: reproduce locally with the same `make` target first. A failing test is never
  "flaky" without proof; fix the root cause. Never skip/disable/quarantine tests, never add lint
  allows or lint-debt entries, never use `--allow-regressions` without a decision record.
- **Eval regressions** in CI (`eval-check`): the PR broke previously-passing Excel behavior. Fix
  the code, do not touch the baseline.
- **Governance failures**: protected paths (CI, .claude/, CLAUDE.md, lint policy, scripts/,
  quality/, docs/process/) changed — add a decision record in docs/decisions/ explaining why, and
  make sure `reviewer-architecture` and `reviewer-security` approved that part explicitly.
- **Stuck**: after 3 failed fix attempts on the same failure, mark the task blocked with details,
  leave the PR as draft, and move on. Do not leave `main` red: if a merge broke `main`, revert first.
