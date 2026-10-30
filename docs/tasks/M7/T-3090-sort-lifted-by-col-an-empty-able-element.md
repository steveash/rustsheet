---
id: T-3090
title: "SORT lifted by_col: an empty-able element below one with an equal by_col when the row's lead is broken"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`SORT(A1:D4,2,1,{TRUE,TRUE;1,TRUE})` matches Excel: row 2 is `#VALUE!`, `#VALUE!`.

## Context
probes/t2947-sort-lifted-by-col-corners Sheet1!L81 (baselined): Excel has `#VALUE!` where `sort_lifted_elements`
(functions/spill_functions.rs) gives an empty element, because the element sits below an empty-able one with the same by_col
while its row's lead (a number 1) is a broken `#VALUE!` element. Requiring no earlier `#VALUE!` in the row broke
t2929-sort-lifted-by-col-errors M2 and N5, so the rule needs more probes (arrays.md, entry "TOROW, TOCOL, SORT and TEXTJOIN
lifted arguments").

## Acceptance criteria
- [x] L81 passes, with probes for the rule's edge cells; t2929 cases still pass.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Sonnet run (2026-10-29) handed off for rule fitting; superseded by the Outcome below.

## Outcome
Excel changed: re-running the unchanged t2366, t2929 and t2947 workbooks on excel-graph (2026-10-29 16:00 UTC, twice, and as
one-formula workbooks) gives different values from the 2026-10-27 and 02:04 UTC fills. Today's rule is much simpler: in row-major
order, the first sorting call keeps its top-left, the next is empty with sort_index 2 (else `#VALUE!`), every later one `#VALUE!`;
no lead comparison, no row keeping, no chain down a column. The seven sidecars were refilled (`fill --force`), the new probe
probes/t3090-sort-lifted-by-col-broken-lead (239 cells, three rounds) pins the corners, and `sort_lifted_elements` was rewritten to
the rule (`ByCol` removed). All eight SORT-lift cases pass (678 cells); L81 passes. arrays.md entry "TOROW, TOCOL, SORT and
TEXTJOIN lifted arguments" rewritten with the drift recorded. Follow-up: T-3153 (re-fill later; text by_col).
