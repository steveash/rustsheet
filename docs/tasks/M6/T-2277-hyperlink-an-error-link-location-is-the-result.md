---
id: T-2277
title: "HYPERLINK: an error link_location is the result, whatever friendly_name is (no spill)"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
HYPERLINK returns an error link_location as its single result, whatever friendly_name is (number, array or range), with no spill, as Excel does.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Rule and cells: docs/excel-semantics/lookup.md "HYPERLINK: an error link_location
is the result" (added by this triage). `fn_hyperlink` (crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs) returns
friendly_name when given, and lifts over an array or range friendly_name even when link_location is an error (so `#SPILL!`, or a
spilled `#N/A` into B214). Check the lift driver's argument kinds for HYPERLINK (lookup.md "probed scalar arguments lift") so that a
lifted *non-error* link_location still lifts.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 4 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/0bcac2d96e96b3df Calc!A17 | `=HYPERLINK(TEXTJOIN(VLOOKUP(Data!$B5,Data!B2:C10,1,Data!E10),(1e-15=Data!D10),Data!C5:E10,Data!$C11),GCD(Da...` | "#VALUE!" | 1.0 |
| gen-m6/44194b2fefa30de2 Calc!A169 | `=HYPERLINK(XLOOKUP(AVERAGE(Data!E7:E7,100),Data!E7:E12,Data!D9:E12),MUNIT(2))` | "#VALUE!" | "#SPILL!" |
| gen-m6/c6621490a4d67f4e Calc!B214 | `(spill cell of A214)` | (empty) | "#N/A" |
| gen-m6/d0c30225243fde60 Calc!A377 | `=HYPERLINK((HLOOKUP(100,Data!A6:C11,7)&(Data!B$9&Data!E8)),OFFSET(Data!A6:C12,6,42))` | "#REF!" | "#SPILL!" |

## Acceptance criteria
- [ ] These 4 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] Unit tests cover `HYPERLINK(NA(),5)`, `HYPERLINK(NA(),{1,2})` (one `#N/A`) and `HYPERLINK("#A1",{1,2})` (still spills).
- [ ] lookup.md's entry says Ours: implemented (T-2277).
- [ ] `make check` green; reviewers approve.

## Out of scope
- HYPERLINK's displayed text and link storage.

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.

## Outcome
`fn_hyperlink` now goes through `lift_call_error_first`: an error link_location is the whole result over a scalar, array or range friendly_name (3 of the 4 cells: gen-m6/44194b2fefa30de2 A169, c6621490a4d67f4e B214, d0c30225243fde60 A377). The fourth, 0bcac2d96e96b3df A17, was not an error location: the TEXTJOIN text is 328 characters and Excel's HYPERLINK gives `#VALUE!` for a link_location past 255 characters (new probe probes/t2277-hyperlink-location-length, excel-graph); implemented too and recorded in lookup.md. Unit tests in test_fn_hyperlink.rs.

## Handoff (resumed)
Escalated by a sonnet run (work-loop step 5b): the architecture reviewer returned REQUEST_CHANGES because the implementation adds an opt-in first-error hook to the lift driver (`lift_call_error_first` / `FirstError`, lift/call.rs and lift/driver.rs), which SPEC-0011 forbids in three places ("the driver never scans for the first error"). Everything else is done and `make check` was green on the pushed branch: behavior, unit tests (test_fn_hyperlink.rs), probe `probes/t2277-hyperlink-location-length` (excel-graph: a link_location past 255 characters is `#VALUE!`; it was the real cause of gen-m6/0bcac2d96e96b3df A17), lookup.md entries, baseline (4 gen-m6 lines removed). Security and quality reviewers APPROVED; excel-fidelity had not reported when this was written.
What is left for opus:
1. Decide the design: either amend SPEC-0011 (an opt-in "error-first hook" paragraph with the HYPERLINK evidence) or a decision record, or avoid a driver change another way (the kernel cannot change the result shape, and collapsing identical element errors afterwards is wrong for an array friendly_name of errors).
2. Make the hook honoured on every argument-count arm of `lift_call_with_first`, or restrict it to exactly two arguments with a `debug_assert!`, consistently with the no-table-entry fallback.
3. Add a lift-level unit test for the hook (counting kernel never called when the check fires).
4. Minor: `hyperlink_of` casts the location twice (make `hyperlink_location_error` return the cast string).
Then re-run the reviewers and open the PR (none is open yet).
