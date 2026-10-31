---
id: T-3533
title: "T.DIST density is #NUM! at a huge x in Excel where the engine returns 0"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`T.DIST(x,df,FALSE)` returns Excel's `#NUM!` where Excel gives it at a huge x, instead of 0.

## Context
- Found by T-3358's Gate A table (excel-graph): `T.DIST(1.01778553276442E+236,2,FALSE)`,
  `T.DIST(1.06518240138577E+269,9,FALSE)`, `T.DIST(3.05696424373289E+292,824733515,FALSE)` and
  `T.DIST(1.43991809793873E+307,20225998,FALSE)` are `#NUM!` in Excel. The true density underflows to 0 there, and the
  engine returns 0. These are Gate A known failures accuracy/T.DIST/s1 A85, A95, A105 and A110.
- statistics.md, "T.DIST's density is #NUM! at a huge x, and T.INV.2T and TINV at p = 2.2E-308 (T-3358)": the cut
  point is not yet probed. T-2752 probes TDIST's x cut, which may share the rule.
- Code: `fn_t_dist` and the density in crates/rustsheet-engine/src/functions/statistical/ (t_dist.rs).

## Acceptance criteria
- [ ] A probe case (M9, excel-graph) brackets the x where the density turns `#NUM!` at several df; the rule is in the
      statistics.md entry.
- [ ] The engine matches it; the four known-failure lines of eval/accuracy/known-failures.tsv are removed as stale.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cumulative T.DIST (unaffected).
