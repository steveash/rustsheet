---
id: T-3090
title: "SORT lifted by_col: an empty-able element below one with an equal by_col when the row's lead is broken"
milestone: M7
lane: functions-lookup
status: todo
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
- [ ] L81 passes, with probes for the rule's edge cells; t2929 cases still pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff
Sonnet run (2026-10-29): no code change. The rule in `sort_lifted_elements` (spill_functions.rs) is an empirical
fit (`broken_seen` / `same_above` / `above`); L81 needs a new condition on rows whose lead is a broken `#VALUE!` element,
and the task already records that "no earlier #VALUE! in the row" broke t2929 M2 and N5. This needs several oracle
probe rounds (excel-graph is OK) to find the rule from edge cells, then a re-fit against t2929 and t2947 together: beyond
a sonnet run (unclear evidence / rule fitting). Next: probe variants of L81 (vary the lead's value, by_col mix, row
position), tabulate against t2929 M2/N5, then adjust the loop.
