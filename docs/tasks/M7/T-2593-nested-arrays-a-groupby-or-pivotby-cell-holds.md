---
id: T-2593
title: "Nested arrays: a GROUPBY or PIVOTBY cell holds its LAMBDA's array result (INDEX, TYPE 64, top-left when lifted; SPEC-0018)"
milestone: M7
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2592, T-2602]
---

## Goal
A GROUPBY or PIVOTBY cell whose function returns an array holds that array, at any depth (SPEC-0018 rules 1-3, 8): it shows `#VALUE!`, INDEX returns the array (TYPE 64), and an element-wise lift takes its top-left (rule 4's first point).

## Context
Implements docs/specs/SPEC-0018-nested-arrays-in-groupby-pivotby-results.md Design "Value" (kind `Array`; `Reference` is T-2594's), "One reader", "Producers" (array results), "Consumers with their own rule" (INDEX, TYPE,
`array_node_to_calc_result`, display, the copying functions of rule 5's last point) and "Bound" (cells and depth). Code: `Model::aggregate` (functions/lookup_and_reference/group_functions.rs)
makes each array result `#VALUE!` today; `ArrayNode` (expressions/parser/mod.rs) beside `ArrayNode::Lambda`, whose 35
matches in 25 files are the inventory of non-scalar reads. Owned known failures:
- probes/t2360-array-lambda-holidays, 47 cells: P!B3:E5, B11:D13, B19:C21, B27:B29, B35:E37 (the INDEX/TYPE grid) and the
  lifted counts P!C1, C9, C17, C25, C33 (`make eval-show CASE=probes/t2360-array-lambda-holidays.case.json`).
- probes/t2311-pivotby-value-columns-error P!C12.
- probes/t2592-nested-groupby-cells, the cells of rules 1, 3 and 8: A16:A18, A22:A24, A26:A41 (whichever of them still fail).
- gen-m7/21b4eef18424ce94, 26 cells of the A1504 spill: Calc!B1505, B1506, B1510, B1512, B1513, C1505, C1507:C1511, C1514,
  D1505, D1507, D1509:D1511, D1513, E1505:E1507, E1509:E1513.
The G! cells of t2360 are blank keys (T-2460), not this task's. PIVOTBY's `""` Total columns over several value
columns stay as they are (SPEC-0018 rule 1's exception; t2360 P!F3:G5 pass today). Reads T-2592 has not shown to take
the top-left keep today's result (SPEC-0018 "Consumers with their own rule").

Handed over from T-2578 (excel-graph, 2026-10-24): four gen-m9 cells over a LAMBDA(x,x)-style function decided by the nested
arrays: gen-m9/06b4e8f5de409d5e Calc!A366 (MAX over PIVOTBY(..,LAMBDA(z,z)) `#N/A`), 68d8ffb69ff739d8 Calc!A285, 7c20351562a2d71d
Calc!A381, e85daedce5c2675e Calc!A35 (HARMEAN, AVERAGE `#N/A`, COUNT 142; arrays.md "GROUPBY and PIVOTBY in the gen-m9 sweep").

## Acceptance criteria
- [ ] The four gen-m9 cells handed over from T-2578 disappear from eval/baseline/gen-m9.txt (or are explained by a remaining cause).
- [ ] The owned cells above pass and leave eval/baseline (`make eval-baseline`), with no new known-failure lines.
- [ ] Unit tests: `NestedArray::new` invariants, `scalar()` through two levels (the assumed rule of SPEC-0018 "One reader", not Excel parity until T-2609 lands) and on a `Reference` (`#VALUE!`), two levels as SPEC-0018 rule 8, the cell cap
      (`#ERROR!` past 1,048,576 kept cells, inner levels included), the depth cap (`MAX_NESTED_DEPTH`, refused in `NestedArray::new`)
      and the walk budget (`MAX_NESTED_WALK_CELLS`) charged by `array_node_to_calc_result` and `PartialEq` (a VSTACK fan-out of a large
      nested cell read through INDEX is `#ERROR!` in bounded time); DEVIATION entries in arrays.md.
- [ ] arrays.md records the implementation (Ours/Status lines); SPEC-0018 status accepted.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MAP, BYROW, BYCOL, REDUCE and the aggregates' own rules (T-2594); SCAN (T-2398); a reference LAMBDA body (rule 7, T-2594).
