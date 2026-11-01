---
id: T-3752
title: "PERCENTOF: probe a 3-D span reached through IF, CHOOSE or a union"
milestone: M8
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
PERCENTOF gives Excel's result when a 3-D reference reaches it indirectly, not only as a bare or parenthesised span.

## Context
T-3653 found that a bare 3-D span in PERCENTOF is a direct `#VALUE!` in its argument's place (probes/t3653-percentof-3d;
numbers.md "PERCENTOF: an error inside a reference or array is #NUM!"). `Model::percentof_sums` (sum.rs) tests the argument
node with `is_multi_sheet_span`, which sees through parentheses only. Not probed: a span through IF (`PERCENTOF(IF(TRUE,S1:S2!A1),1)`),
through CHOOSE, inside a union (`(S1:S2!A1,S1!A2)`), and a defined name whose body is a span. `is_span_or_if_of_span`
(expressions/parser/sheet_span.rs) already covers the IF form if the oracle says Excel refuses it too.

## Acceptance criteria
- [ ] A probe case (excel-graph, tagged M8) covers the four forms above, each also paired with a direct error before and after it.
- [ ] The engine matches every probed cell, and numbers.md's PERCENTOF entry states the result for each form.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SUM and the other aggregates over 3-D spans.
