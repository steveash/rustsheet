---
id: T-3702
title: "Review panel by diff kind: security only on executable changes, one design-and-quality reviewer"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Each PR gets the reviewers its diff can use: the security reviewer runs only when the diff has an
executable change, and one design-and-quality reviewer replaces the separate architecture and
quality reviewers. Excel fidelity is unchanged.

## Context
Owner request (2026-11-01, CI-speed and review-cost investigation). Parsed "Agent review" tables
of 271 of the last 300 merged PRs (heuristic: a reviewer counts when it requested changes or a
later round fixed its finding): excel-fidelity caught something in about 52 of about 190 applicable
PRs (27%), quality in about 32 of 265 (12%, mostly duplicated helpers), architecture in about 20
(7%, about half task-file or process wording), security in about 8 (3%), of which about 4 were
substantive and all on code changes (#2564 quadratic UNIQUE walk, #2571 i32 overflow, #2395 markup
injection). Every reviewer round costs tokens and usually a push and a CI run.

## Acceptance criteria
- [x] A decision record (amending 0015 and 0081's panel) defines "executable change" by path
      (at least `crates/`, `bindings/`, `scripts/`, `tools/`, `.github/`, `Makefile`, `Cargo.*`,
      `deny.toml`) and the merged reviewer's mandate (the union of architecture's and quality's
      blocking rules).
- [x] `.claude/skills/review/SKILL.md`, `docs/process/review-policy.md`, the agent files and the
      verdict-record tooling (`scripts/merge_ready.py` REVIEWERS, `make mark-ready`) agree on the new
      panel; old verdict records stay readable during the switch.
- [x] `make check` green; reviewers approve (on the old panel, for this PR).

## Out of scope
- Excel-fidelity's mandate or model; the performance reviewer (decision 0081).

## Outcome
Decision 0236 sets a new panel.
- **Design.** `reviewer-design` (Sonnet) replaces `reviewer-architecture` and
  `reviewer-quality`. Its prompt keeps every check and blocking rule of both.
- **Security.** `reviewer-security` reviews every diff unless every path is under `docs/` or
  `eval/suites/` and none is under `docs/process/`. Unknown or empty path lists keep the full
  panel, as the architecture reviewer asked on #2729.
- **Excel fidelity** is unchanged.

The panel's one copy outside the `review` skill moved out of `scripts/merge_ready.py` into
`scripts/review_panel.py`, keeping merge_ready.py under the 1,000-line ratchet.
`scripts/test_merge_ready.py` checks the module against the skill's text for the always-on
reviewers, security's skip and keep paths, and excel-fidelity's paths.

Transition: markers and verdict records that name `architecture` or `quality` still parse, and
both approving counts as `design` (tested in test_merge_ready and test_verdict_records). PRs
marked under the old panel stay ready.

Updated: the review, steward and work-loop skills, review-policy.md, agent-workflow.md,
SPEC-0027's examples, the PR template, the Makefile help, and `make verdict-record`'s reviewer
list.
