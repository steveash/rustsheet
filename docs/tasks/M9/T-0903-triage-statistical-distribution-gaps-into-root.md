---
id: T-0903
title: "Triage statistical-distribution gaps into root-cause tasks"
milestone: M9
lane: research
status: done
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
- [x] Tasks filed (functions lane, S/M each) with failing case lists and suspected algorithms; this task closed with links.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Triage of the 2026-10-06 report: 680 failing distribution cells in `ironcalc-calc` (no gen/probes
cells), grouped by kernel. Already owned: T-0925 inverse normal (106), T-0927 discrete pmf (143).
Filed (functions-stat unless noted):
- T-0969 Student-t CDF, direct upper tail (55); T-0967 Student-t quantile (89, after T-0969 + T-0966)
- T-0970 regularized incomplete beta (45); T-0974 inverse incomplete beta (28, after T-0970 + T-0966)
- T-0971 regularized incomplete gamma (43); T-0972 gamma quantile (39, after T-0971 + T-0966)
- T-0973 gamma function: GAMMA + GAMMALN residue (39)
- T-0968 normal CDF / erfc residue (59, after T-0966); T-0975 normal densities + PHI (24)
- T-0976 FISHERINV formula (6, sonnet); T-0977 HYPGEOM.DIST (2, after T-0966); T-0978 WEIBULL.DIST (2, after T-0966)
- T-0966 (research) oracle sweeps per kernel family plus evidence for the 28 legacy aliases.
Quirks recorded in `docs/excel-semantics/statistics.md` (new) and `numbers.md` (GAMMALN).
Also filed with this PR: T-0250 (research; T-0211 review follow-up, VLOOKUP/HLOOKUP index-shape probe).
