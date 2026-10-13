---
id: T-0287
title: "List the WEEKNUM stale return_type cells as accepted deviations (decision 0038)"
milestone: M2
lane: harness
status: dropped
priority: P2
size: S
tier: opus
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

## Handoff (resumed)
Superseded by the Outcome below: the task is dropped, no decision amends 0036.
- Tried (sonnet run, PR #394, draft): listed the five cells in `eval/suites/probes/deviations` (decision 0038, dates.md
  anchor `weeknum-return_type-edge-cases-and-serial-0-t-1023`, pinned `#NUM!`), made the dates.md entry's status line start
  `Status: DEVIATION` (the parser requires it), ran `make eval-baseline` (removes the five F lines). All five cells were
  checked as the stale-state case; `make check` was green; security, quality and architecture-style mechanics were fine.
- Blocked: the architecture reviewer found the change contradicts accepted decisions. Decision 0038 (Consequences) says
  the cells stay F lines and "do not go into a `deviations` file: decision 0036 limits that file to directive 5
  deviations, and moving them would need a decision that amends 0036". 0036 also has the excel-fidelity reviewer check
  that a listed limit is a directive-5 limit. A task file cannot override those, and the change moves the cells out of the
  M2 exit denominator (decision 0043).
- Next: an opus run writes a decision (`make new-decision`, protected path) that amends 0036 to let `deviations` hold
  calc-order-dependent cells (Excel's value is not a function of the workbook, 0038/0033), states how the M2 exit rate
  counts them, and updates 0038's Consequences; or drops this task. Then reuse the branch
  `agent/harness/T-0287-weeknum-deviations` (the mechanical change is done), and split the dates.md status line as the
  reviewer suggested ("Status: DEVIATION (decision 0038) for the stale state." then a separate sentence that the rest is VERIFIED).

## Outcome (dropped, 2026-10-08, opus)
Dropped rather than amending decision 0036. The five cells differ from Excel because RustSheet does
not model WEEKNUM's calc-order-dependent stale state (decision 0038). They do not differ because of
a directive-5 limit. CLAUDE.md directive 1 allows deviations only for directive 5, and decision 0036
restricts the `deviations` file to exactly those (its excel-fidelity check is "a directive-5 limit,
never a convenience"). Listing these cells would take them out of the M2 exit denominator
(decision 0043) without a directive-5 reason, which weakens an exit gate. Decision 0038's
Consequences already settle this: they stay F lines in eval/baseline/probes.txt, as decision 0033's
cells do, and decision 0043 already names them in M2 scope. The sonnet run's mechanical change
(draft PR #394) is not merged. Revisit only together with 0038's revisit trigger (RustSheet
reproducing Excel's calc order).
Superseded by decision 0052 (2026-10-13, T-1030): decision 0038 is no longer a deviation, so these
cells are a known gap that T-1484 specifies, never `deviations` lines.
