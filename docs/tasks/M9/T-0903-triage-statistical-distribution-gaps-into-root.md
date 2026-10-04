---
id: T-0903
title: "Triage statistical-distribution gaps into root-cause tasks"
milestone: M9
lane: research
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Group the ~600 failing cells across NORM/BINOM/GAMMA/BETA/T/POISSON/LOGNORM/NEGBINOM families by shared numerical kernel (gamma, incomplete beta, erf, inverse normal) and file one task per kernel.

## Context
Run the gap-triager agent; see eval/runs/latest/report.md.

## Acceptance criteria
- [ ] Tasks filed (functions lane, S/M each) with failing case lists and suspected algorithms; this task closed with links.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
