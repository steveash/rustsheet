---
id: T-3390
title: "Tighten the T-3349 statistics.md entry to its evidence (16-digit literal, Excel-internals phrasing, TINV and CONFIDENCE.T scope)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The T-3349 entry in docs/excel-semantics/statistics.md claims only what its evidence shows.

## Context
- PR #2527's excel-fidelity review (minors): the entry cited an unprobed 16-digit literal, described Excel's
  internals ("solves the centre directly"), and stated the rule for TINV and CONFIDENCE.T at every df from 2 where the
  probe has them at df 2 only. The T-3349 task file left an acceptance box unticked while done.

## Acceptance criteria
- [x] The entry cites numbers.md for the 15-digit literal and names A38 as `T.INV.2T(1,2)`, words the rule as an
      observation, and limits TINV and CONFIDENCE.T to the probed df 2; T-3349's boxes say what was met.
- [x] `make check` green; reviewers approve.

## Out of scope
- New probes.

## Outcome
Docs only: statistics.md entry and T-3349's task file reworded; eval delta none.
