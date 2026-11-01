---
id: T-3702
title: "Review panel by diff kind: security only on executable changes, one design-and-quality reviewer"
milestone: M0
lane: harness
status: todo
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
- [ ] A decision record (amending 0015 and 0081's panel) defines "executable change" by path
      (at least `crates/`, `bindings/`, `scripts/`, `tools/`, `.github/`, `Makefile`, `Cargo.*`,
      `deny.toml`) and the merged reviewer's mandate (the union of architecture's and quality's
      blocking rules).
- [ ] `.claude/skills/review/SKILL.md`, `docs/process/review-policy.md`, the agent files and the
      verdict-record tooling (`scripts/merge_ready.py` REVIEWERS, `make mark-ready`) agree on the new
      panel; old verdict records stay readable during the switch.
- [ ] `make check` green; reviewers approve (on the old panel, for this PR).

## Out of scope
- Excel-fidelity's mandate or model; the performance reviewer (decision 0081).
