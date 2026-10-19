---
id: T-1955
title: "Spec: an array-of-references result for OFFSET, INDIRECT and TRIMRANGE lifts"
milestone: M6
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A spec in docs/specs/ settles how the evaluator carries a lifted call whose elements are references (OFFSET, INDIRECT, TRIMRANGE over a multi-cell range or an array), so the implementation tasks T-1956 and T-1957 can start.

## Context
Split from T-1810. The evidence (docs/excel-semantics/lookup.md "OFFSET, INDIRECT and TRIMRANGE lift into arrays of references" (probes/reference-lift-elements, excel-graph, 2026-10-19)) shows the elements are references, not values: spilled directly each is `#VALUE!` (an error element stays itself); INDEX picks an element as the reference itself (a one-cell pick is its value and ISREF TRUE, a multi-cell pick spills its cells or is `#SPILL!`); ROWS/COLUMNS count elements; ISREF of the whole is one FALSE; SUM reads each element (a multi-cell element apparently only its first cell, one probe cell). `ArrayNode` holds values only (SPEC-0011 non-goal 2, open question 5), and INDEX's own index lift yields values (lookup.md "INDEX lifts over array and range indices"), so neither carries this. The spec weighs a reference-array `CalcResult` variant against a lazily evaluated lifted node INDEX/ROWS/SUM can see through, lists every consumer that must handle it (spill, INDEX, ROWS/COLUMNS, ISREF, SUM and the other aggregates, operators), the resource bounds (directive 5: element count, nesting), and names the probes still needed (SUM of a multi-cell element, COUNT, operators over the result). Build on `lift_values`/`read_lift_arg` (functions/lift/).

## Acceptance criteria
- [ ] docs/specs/SPEC-NNNN-*.md (accepted by the reviewers) picks one carrier, with the consumer list, the bounds and the open probe questions.
- [ ] T-1956 and T-1957 are updated to cite it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The implementation (T-1956, T-1957).
