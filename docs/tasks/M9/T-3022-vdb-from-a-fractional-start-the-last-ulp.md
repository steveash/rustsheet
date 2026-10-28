---
id: T-3022
title: "VDB from a fractional start: the last ulp of 8 probe cells with a start fraction of .2 or .3"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
VDB switching from a fractional start period returns Excel's doubles to the bit on the 8 probe cells T-2185 left one ulp
off.

## Context
T-2185 fitted VDB's switching schedule from a fractional start (docs/excel-semantics/financial.md "VDB charge order":
the periods are laid on the start's fraction). It matches 153 of 161 switching-mode cells to the bit and the other 8 to
one ulp. All 8 have a start fraction of .2 or .3, where `start - floor(start)` is not exact (6.3 - 6 is
0.29999999999999982), so the first partial charge `fraction·min(cost·rate, cost - salvage)` is the likely source:
- probes/fin-vdb-fractional-start A1 (`VDB(2400,300,10,6.3,7.3,1.5)`: Excel 152.54735664893622, ours
  152.5473566489362), A4, A6, A11, A14, A26, A29, A35;
- gen-m9/762cd0e4ec9a7aa9 Calc!A379 and probes/t2492-vdb-ddb-arguments A1 (start 0.2) pass, so the fraction alone is
  not the whole story.
Try other ways to form the first grid point (book from `cost - start·charge` for a start under 1, the fraction as
`start - trunc(start)` against `start mod 1`, the book stepped from the start backwards), keeping the other 153 exact.
The Python model of T-2185 (described in its Outcome) reproduces the engine's values and is quick to vary.

## Acceptance criteria
- [ ] The 8 lines of probes/fin-vdb-fractional-start in eval/baseline/probes.txt disappear (`make eval-baseline`),
      with no new known-failure lines in any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The no_switch TRUE path.

## Deferred (decision 0114)
`defer: phase-1b`: the 8 cells are within one ulp of Excel, so they are not visibly off and wait until Phase 1 exits.
