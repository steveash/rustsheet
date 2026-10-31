---
id: T-3386
title: "T.INV.2T and TINV at a tiny p: Excel is over 1e-4 off the math from p 1.7E-110 down (R3 candidate, needs an owner ruling)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-3358]
---

## Goal
Decision 0188 R3 decides, with an owner ruling, whether T.INV.2T and TINV at a tiny p follow Excel or the math, and
the region is evidenced as committed points; until then Excel's value stays the target (directive 1).

## Context
- Gate A known failures this task owns (T-3358, eval/accuracy/known-failures.tsv): accuracy/T.INV.2T/s1 A68 and A78, and accuracy/TINV/s1 A68 and A78 (Excel #NUM! at p = 2.2E-308; error mismatches, which R3 never excuses). The R3 candidates of this task's region are T.INV.2T/s1 A98, A99 (p 4.4E-242) and A103 and TINV/s1 A102, which are not marked (decision 0213). Removing the lines
  once they pass is part of this task (a passing listed point fails the check as stale).
- T-3349's fresh sweep (probes-numeric/t3349-t-inv-2t-sweep, excel-graph 2026-10-31, mpmath 60 digits): 42 of 300
  points have Excel more than 1e-4 relative off the math, every one at p from 1.38E-300 to 1.73E-110 and df from
  2490 to 1.3E8 (A211-A300, extreme and log-uniform strata). Example `T.INV.2T(6.68863396408457E-199,2490)`: Excel
  100.83724584548176, math 33.047211122914894, engine 33.047211122914874. The engine returns the math there, so these
  cells are visibly off Excel but inside Gate A's envelope.
- R3's initial case 1 lists T.INV (left tail), F.INV, BETA.INV and CHISQ.INV.RT below p 1e-150, not T.INV.2T or
  TINV, and these points reach p 1.7E-110, above that bound. A new R3 case or a wider region needs a decision record
  with an owner ruling (decision 0188 R3); no agent adds one alone.
- docs/excel-semantics/statistics.md "Two-tailed Student-t quantile with p near 1 is accurate in Excel (T-3349)".

## Acceptance criteria
- [ ] A decision record with the owner's ruling (follow the math, or match Excel) and the region's bounds, from
      committed evidence points (T-3358's tables or a probe on both sides of the bound).
- [ ] If the ruling is the math: a `MATH-OVER-EXCEL` entry and the R3 marks (SPEC-0029); if Excel: a task that
      reproduces Excel's form there.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T.INV's own region (R3 case 1, T-3358).
