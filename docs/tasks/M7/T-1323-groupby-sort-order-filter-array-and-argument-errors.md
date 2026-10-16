---
id: T-1323
title: "GROUPBY sort_order, filter_array and argument errors (SPEC-0006 step 2)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-1322]
spec: docs/specs/SPEC-0006-groupby-pivotby.md
---

## Goal
GROUPBY's sort_order, filter_array and invalid-argument errors match the probe.

## Context
SPEC-0006 GROUPBY bullets sort_order / filter_array; anchors G!A145, G!J145, G!S145, G!AB145, G!A169, G!S217, G!AB217, G!A241, G!A217, G!J241. Sort by value columns pins the total rows and keeps blank keys last.

From T-1322 (GROUPBY core landed in functions/lookup_and_reference/groupby.rs): `Grouping::sort_by_keys` already takes a
direction per key field (blank last both ways, numbers exact); sort_order, filter_array and field_relationship are
`#N/IMPL!` today (the `given(...)` check in `Model::groupby`). Also score with this step:
- gen-m9/3ec66461400faf2f Calc!A17 (Excel `#VALUE!`, engine `#N/IMPL!`): `GROUPBY(Data!C1:D12,Data!C1:E12,LAMBDA(w,w),1,1,Data!C6:C6,(Data!D1:D12>1E15))`, an all-FALSE filter.
- probes/groupby-blank-values G!J49 and its AC49/AD49 companions (sort_order -1 with an empty-text key: "a", "", 0, Total).

## Acceptance criteria
- [x] Those anchors pass (in their LAMBDA form until T-1325 lands) and leave the baseline.
- [x] `make check` green
- [ ] reviewers approve.

## Out of scope
-

## Outcome
groupby.rs gains the pieces PIVOTBY (T-1326) reuses: `SortOrder` (`Keys(Vec<bool>)` or `Value { column, ascending }`)
read by `Model::sort_order(arg, key_columns, value_columns, cell)`; `Grouping::sort_by_aggregate(ascending, cell,
aggregate)`, a stable, hierarchical value sort (parents by the aggregate of their rows, children inside them, ties in key
order, one call per group and parent) that returns the sorted values so they are not computed twice;
`Model::filter_rows(filter, height, first, cell)`, the kept source rows; and `Model::evaluate_options(args, cell)`, which
evaluates the optional arguments up front and reports the first error value in argument order. GROUPBY now validates
every argument after those error values and only then reports `#N/IMPL!` (function arrays, subtotals, an array
field_relationship, a table sorted by value). Scalar field_relationship: 0 is the default, 1 is `#VALUE!` with total_depth
±2 and the default layout otherwise; anything else is `#VALUE!`.
New probe probes/groupby-sort-filter-args (83 questions, excel-graph, 2026-10-16): fractional, zero, text, TRUE and blank
sort_order are `#VALUE!`; arrays mixing key and value columns are `#VALUE!`; value ties keep the ascending key order in
both directions; filter elements count like conditions (numbers, blanks, "TRUE"/"FALSE" text), other text and error
elements are `#VALUE!`; with headers the filter may cover the header row or only the records; error values win over
invalid arguments, first in argument order. Recorded in docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY" and
SPEC-0006. Review follow-up probe probes/groupby-sort-filter-args2 (37 questions, excel-graph): the filter element
beside a header row is ignored whatever it holds (now implemented), "true"/"True" count, "1" does not, `{#N/A}` is
`#VALUE!` in sort_order and filter_array, a one-cell error reference is that error, three-key value sorts are
hierarchical, parents rank by the function of all their rows (AVERAGE, `100-SUM`), booleans sort between text and errors
(all already matched).
Eval delta (passing cells): probes/groupby-pivotby 360 -> 433 (+73), groupby-pivotby-followup 289 -> 716 (+427),
followup2 162 -> 239 (+77), followup3 407 -> 472 (+65), groupby-blank-values 100 -> 110 (+10, now Pass),
groupby-sort-filter-args 84/84 (new), gen-m9/3ec66461400faf2f Calc!A17 passes (gen-m9 is not gated). Still failing in
these cases: total_depth ±2 and an array field_relationship (T-1324), function arrays and a one-cell reference LAMBDA
result (T-1325).
