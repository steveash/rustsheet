---
id: T-1849
title: "BETA.DIST, F.DIST, F.DIST.RT and FDIST read a TRUE as 1 in their numeric arguments"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
BETA.DIST (and BETADIST), F.DIST, F.DIST.RT and FDIST read a typed TRUE as 1 in their numeric
arguments, directly and as an array element, as Excel does, instead of `#VALUE!`.

## Context
Found by T-1290, which lifted these functions (SPEC-0011) without changing their scalar reads:
- probes/stat-later-argument-lift L1 `ROUND(INDEX(BETA.DIST({0.5;TRUE},2,3,TRUE),2),12)` is 1 in Excel (ours
  `#VALUE!`), and L13 `ROUND(INDEX(F.DIST({1.5;TRUE},5,5,TRUE),2),12)` is 0.5 (ours `#VALUE!`), excel-graph 2026-10-17.
- probes-numeric/t1049-fdistrt A19 `F.DIST.RT(TRUE,3,7)` = 0.4470796134684837 and A26 `F.DIST.RT(1,TRUE,TRUE)` =
  0.4999999999999999; probes-numeric/m9-legacy-fdist A90 `FDIST(TRUE,2,5)` = 0.43120115037169215.
The kernels read these arguments with `cast_number_no_bools_ref` (statistical/beta.rs, statistical/fisher.rs, the
`TODO(T-1849)` lines). STANDARDIZE's TRUE is T-1755 (and probes/stat-later-argument-lift L14). Before changing a
function whose boolean read is unprobed (BETA.DIST's alpha, beta, A, B; F.DIST's x directly; WEIBULL, PHI, HYPGEOM.DIST,
BETA.INV, F.INV, F.INV.RT), probe it (excel-probe skill); docs/excel-semantics/statistics.md "Statistical function
argument modes" records the rule.

## Acceptance criteria
- [ ] probes/stat-later-argument-lift L1 and L13 pass (their baseline lines removed), and the probes-numeric cells above.
- [ ] Each changed argument cites its probe cell in a code comment; the rule is in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
