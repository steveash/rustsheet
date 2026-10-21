---
id: T-2260
title: "Probe PHI's #NUM! cut between 38.5 and 1E300 (t1049-phi A25, A26)"
milestone: M9
lane: research
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's rule for where PHI turns from an underflowed 0 into `#NUM!` is in eval and recorded in
docs/excel-semantics/statistics.md, so the two visibly-off PHI cells can be fixed against evidence.

## Context
Gap triage 2026-10-21 (decision 0083 visibly-off cells). probes-numeric/t1049-phi (excel-graph, ungated suite):
`PHI(38.5)` and `PHI(-38.5)` are 0 (A20, A21, pass), but `PHI(1E300)` and `PHI(-1E300)` are `#NUM!` (A25, A26) where
ours gives 0. statistics.md (T-1049 "Thin distribution functions" entry, line ~2001) records both and T-2035 leaves
A25/A26 out of scope because the cut is unprobed. A plausible lead, to test rather than assume: Excel's `z*z`
overflows to infinity past about 1.34E+154, and an infinite intermediate is `#NUM!`; another is an explicit argument
bound. Code: `phi.rs` under crates/rustsheet-engine/src/functions/statistical/.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph, tag M9) bisects the cut for both signs: PHI at 1E+100, 1E+150,
      1E+154, 1.3407807929942596E+154 and the next doubles around it (`sqrt(DBL_MAX)`), 1E+155, 1E+200, and the same
      points for `NORM.S.DIST(z,FALSE)` and `NORM.DIST(z,0,1,FALSE)` to see whether the kernels share the cut.
- [ ] statistics.md states the rule (VERIFIED) with the cells.
- [ ] If the rule is a one-comparison change in `phi.rs`, it lands in this PR and probes-numeric/t1049-phi A25 and
      A26 pass in `make eval` with no new known-failure lines in any baseline; otherwise a follow-up functions-stat
      task with `campaign: visible` owns A25 and A26.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PHI of booleans (T-2035).
