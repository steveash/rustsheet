---
id: T-2593
title: "Nested arrays: a GROUPBY or PIVOTBY cell holds its LAMBDA's array result (INDEX, TYPE 64, top-left when lifted; SPEC-0018)"
milestone: M7
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2592]
---

## Goal
A GROUPBY or PIVOTBY cell whose function returns an array holds that array (SPEC-0018 rules 1-4): it shows `#VALUE!`, INDEX returns the array (TYPE 64), and a lifted read takes its top-left.

## Context
Implements docs/specs/SPEC-0018-nested-arrays-in-groupby-pivotby-results.md Design "Value", "One reader", "Producers", "Consumers with their own rule" (INDEX, TYPE,
`array_node_to_calc_result`, display) and "Bound". Code: `Model::aggregate` (functions/lookup_and_reference/group_functions.rs)
makes each array result `#VALUE!` today; `ArrayNode` (expressions/parser/mod.rs) beside `ArrayNode::Lambda`, whose 35
matches in 25 files are the inventory of non-scalar reads. Owned known failures:
- probes/t2360-array-lambda-holidays, 47 cells: P!B3:E5, B11:D13, B19:C21, B27:B29, B35:E37 (the INDEX/TYPE grid) and the
  lifted counts P!C1, C9, C17, C25, C33 (`make eval-show CASE=probes/t2360-array-lambda-holidays.case.json`).
- probes/t2311-pivotby-value-columns-error P!C12.
- gen-m7/21b4eef18424ce94, 26 cells of the A1504 spill: Calc!B1505, B1506, B1510, B1512, B1513, C1505, C1507:C1511, C1514,
  D1505, D1507, D1509:D1511, D1513, E1505:E1507, E1509:E1513.
The G! cells of t2360 are blank keys (T-2460), not this task's. PIVOTBY's `""` Total columns over several value
columns stay as they are (SPEC-0018 rule 1's exception; t2360 P!F3:G5 pass today). Reads T-2592 has not shown to take
the top-left keep today's result (SPEC-0018 "Consumers with their own rule").

## Acceptance criteria
- [ ] The owned cells above pass and leave eval/baseline (`make eval-baseline`), with no new known-failure lines.
- [ ] Unit tests: `NestedArray::new` invariants, `scalar()`, two levels flattened, and the cap (`#ERROR!` past 1,048,576 kept cells; DEVIATION
      entry in arrays.md).
- [ ] arrays.md records the implementation (Ours/Status lines); SPEC-0018 status accepted.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MAP, SCAN, REDUCE and the aggregates' own rules (T-2594); a reference LAMBDA body (rule 7).
