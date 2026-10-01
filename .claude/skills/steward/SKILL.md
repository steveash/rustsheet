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
- **Generated files**: never hand-edit `eval/baseline/*.txt`, `quality/*.json`, `docs/tasks/BOARD.md`;
  regenerate with `make eval-baseline`, the `scripts/check_*.py --update` commands, `make board`.
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
