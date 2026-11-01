---
id: T-3554
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
- T-3425's batch adds three: accuracy/T.DIST/e-T-3425 A116 (`T.DIST(4.20674629135061E+220,2.42593594707487E+249,FALSE)`)
  and, at a negative x, A103 (`T.DIST(-4.62236657470614E+224,11,FALSE)`) and A90 (`T.DIST(-1.3895161258545E+237,16,FALSE)`).
  A negative x is in scope: its cut lies between |x| 7.6E+87 and 4.6E+224.
- statistics.md, "T.DIST's density is #NUM! at a huge x, and T.INV.2T and TINV at p = 2.2E-308 (T-3358)": the cut
  point is not yet probed. T-2752 probes TDIST's x cut, which may share the rule.
- Code: `fn_t_dist` and the density in crates/rustsheet-engine/src/functions/statistical/ (t_dist.rs).

## Acceptance criteria
- [ ] A probe case (M9, excel-graph) brackets the x where the density turns `#NUM!` at several df, for positive and
      negative x; the rule is in the statistics.md entry.
- [ ] The engine matches it; the seven known-failure lines of eval/accuracy/known-failures.tsv are removed as stale.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cumulative T.DIST (unaffected).
