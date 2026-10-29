---
id: T-3090
title: "SORT lifted by_col: an empty-able element below one with an equal by_col when the row's lead is broken"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
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
- [ ] L81 passes, with probes for the rule's edge cells; t2929 cases still pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
