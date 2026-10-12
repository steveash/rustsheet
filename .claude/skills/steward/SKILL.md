---
name: steward
description: Repo-specific conventions for driving RustSheet PRs to merge autonomously - CI failure handling, review comments, merge policy. Read before acting on CI or review events on a RustSheet PR.
---
# PR stewardship conventions (RustSheet)

- **Merge policy**: no human approval is required. A PR may be squash-merged by the agent that
  opened it when (a) all required CI jobs are green on the latest head, (b) the `review` skill
  table shows APPROVE from every applicable reviewer, (c) the branch is up to date with `main`.
  Check (c) mechanically right before every merge call (decision 0028): `git fetch origin main
  <branch>`, then `git merge-base --is-ancestor origin/main origin/<branch>`, and pass that same
  `origin/<branch>` sha as `expectedHeadSha`. If it fails, `main` moved since
  the last CI run, and the green CI does not cover the merged result. GitHub's `mergeable_state:
  clean` only means "no textual conflict", and two PRs that each add `T-0129` (or decision `0026`)
  never conflict textually. Merge `origin/main` in, run `make check`, push, and merge only after
  CI is green on that new head.
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
- **Sonnet queue buffer** (decision 0021): in the steward's queue step, also run
  `make ready-tasks TIER=sonnet`; below 10 ready, run the gap-triager and/or planner to cut
  sonnet-sized tasks by the tier rubric (never relabel a task to reach the number).
- **Generated files**: never hand-edit `eval/baseline/*.txt` or `quality/*.json`; regenerate with
  `make eval-baseline` and the `scripts/check_*.py --update` commands. `docs/tasks/BOARD.md` is
  git-ignored and never committed (decision 0017).
- **CI failures**: reproduce locally with the same `make` target first. A failing test is never
  "flaky" without proof; fix the root cause. Never skip/disable/quarantine tests, never add lint
  allows or lint-debt entries, never use `--allow-regressions` without a decision record.
- **Nightly fuzz failures** (`.github/workflows/fuzz.yml`, decision 0035): a red run is a
  directive-5 bug, not a flake. File one P0 task per distinct crash (dedupe by panic site/stack
  against open tasks) per docs/process/agent-workflow.md "When something goes wrong".
- **Eval regressions** in CI (`eval-check`): the PR broke previously-passing Excel behavior. Fix
  the code, do not touch the baseline.
- **Accepted deviations** (decision 0036): a suite's `deviations` file lists cells that differ
  from Excel by design. "100% bit-exact" in an exit criterion means 100% of the scored cells
  outside the report's `deviations` column. A stale-line warning (the cell now matches Excel)
  means the line goes in the next PR; a listed cell with any value but its pinned one fails the
  gate like any regression.
- **Frozen probe lists** (decision 0043): M1's probe criterion is `eval/milestones/m1-probes.txt`,
  never the whole `probes` suite. Read the list's known failures from the "Milestone probe lists"
  section of `eval/runs/latest/report.md` after `make eval` (also printed by `make eval-check`;
  decision 0044). While the list has failures, put the fixing lanes (engine, then harness) first
  on the `**Critical lane:**` line, ahead of research. Never add a line to a frozen list.
- **Probe milestones** (decision 0051): every other probe case counts toward the milestone that
  decision 0051 assigns it, and only M0–M2 cases count toward M2 exit. When you choose the
  Critical lane and report probe failures in docs/status.md, use the M0–M2 count, not the whole
  suite's. Until T-1416 adds that count to the report, get it from the one-liner in decision
  0051, "Measurement".
- **Governance failures**: protected paths (CI, .claude/, CLAUDE.md, lint policy, scripts/,
  quality/, docs/process/, eval/milestones/) changed — add a decision record in docs/decisions/
  explaining why, and make sure `reviewer-architecture` and `reviewer-security` approved that part explicitly.
- **Stuck**: after 3 failed fix attempts on the same failure, mark the task blocked with details,
  leave the PR as draft, and move on. Do not leave `main` red: if a merge broke `main`, revert first.
