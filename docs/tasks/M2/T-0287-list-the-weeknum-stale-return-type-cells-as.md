---
id: T-0287
title: "List the WEEKNUM stale return_type cells as accepted deviations (decision 0038)"
milestone: M2
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The five `probes/weeknum-stale-return-type` cells that differ from Excel by design score as
accepted deviations (decision 0036), not as known failures, so M2 exit (decision 0043) does not
count them.

## Context
Decision 0038 declines to model WEEKNUM's stale return_type state (Excel reuses the start weekday
of the last valid WEEKNUM evaluated earlier); docs/excel-semantics/dates.md marks it
`DEVIATION (decision 0038)`. But the cells are still F lines in eval/baseline/probes.txt:
`Mixed!A3`, `Mixed!A4`, `Type1!A1`, `Type21!A1`, `Up!A1`. Decision 0036's mechanism is a line per
cell in `eval/suites/probes/deviations` (case, cell, decision, DEVIATION anchor, pinned value);
see `crates/rustsheet-eval/src/deviations.rs` for the format and the existing entries.

## Acceptance criteria
- [ ] Each of the five cells is listed in `eval/suites/probes/deviations` with decision 0038, the
      dates.md DEVIATION anchor and RustSheet's current value as the pinned value; check every cell
      really is the stale-state case (not a separate bug) before listing it.
- [ ] `make eval-baseline` removes the five F lines and the report counts them as deviations;
      no other change.
- [ ] `make check` green; reviewers (excel-fidelity included) approve.

## Out of scope
- Modelling the stale state.

## Outcome
Listed `Up!A1`, `Mixed!A3`, `Mixed!A4`, `Type21!A1` and `Type1!A1` of probes/weeknum-stale-return-type in
`eval/suites/probes/deviations` (decision 0038, dates.md "WEEKNUM return_type edge cases and serial 0 (T-1023)", pinned
`#NUM!`). Each was checked: the oracle gives a number only because a valid WEEKNUM (type 2, 3, 21 or the default) sits
below it and ran first (Excel evaluates bottom to top on load). The entry's status line now starts `Status: DEVIATION`
(the eval parser requires it). Eval delta: the five F lines left eval/baseline/probes.txt and score as deviations;
0 regressions.
