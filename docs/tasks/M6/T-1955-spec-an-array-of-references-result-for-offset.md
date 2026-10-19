---
id: T-1955
title: "Spec: an array-of-references result for OFFSET, INDIRECT and TRIMRANGE lifts"
milestone: M6
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A spec in docs/specs/ settles how the evaluator carries a lifted call whose elements are references (OFFSET, INDIRECT, TRIMRANGE over a multi-cell range or an array), so the implementation tasks T-1956 and T-1957 can start.

## Context
Split from T-1810. The evidence (docs/excel-semantics/lookup.md "OFFSET, INDIRECT and TRIMRANGE lift into arrays of references" (probes/reference-lift-elements, excel-graph, 2026-10-19)) shows the elements are references, not values: spilled directly each is `#VALUE!` (an error element stays itself); INDEX picks an element as the reference itself (a one-cell pick is its value and ISREF TRUE, a multi-cell pick spills its cells or is `#SPILL!`); ROWS/COLUMNS count elements; ISREF of the whole is one FALSE; SUM reads each element (a multi-cell element apparently only its first cell, one probe cell). `ArrayNode` holds values only (SPEC-0011 non-goal 2, open question 5), and INDEX's own index lift yields values (lookup.md "INDEX lifts over array and range indices"), so neither carries this. The spec weighs a reference-array `CalcResult` variant against a lazily evaluated lifted node INDEX/ROWS/SUM can see through, lists every consumer that must handle it (spill, INDEX, ROWS/COLUMNS, ISREF, SUM and the other aggregates, operators), the resource bounds (directive 5: element count, nesting), and names the probes still needed (SUM of a multi-cell element, with values that tell "first cell only" from the alternatives, e.g. A1 = 1 and A2 = 100, since 1+1 = A2 in the first probe; COUNT; operators over the result). Build on `lift_values`/`read_lift_arg` (functions/lift/). SPEC-0011 "Elements that are references" (T-1833) reads a lifted TAKE's or DROP's whole element (probes/m7-argument-modes H4:I6, K1-K4) as a reference too, so the carrier also serves TAKE and DROP (T-1838): one definition for all five producers.

## Acceptance criteria
- [x] docs/specs/SPEC-NNNN-*.md (accepted by the reviewers) picks one carrier, with the consumer list, the bounds and the open probe questions.
- [x] T-1956 and T-1957 are updated to cite it, and SPEC-0011's open question 5 (lifted elements that are references) points to the new spec.
- [x] `make check` green; reviewers approve.

## Out of scope
- The implementation (T-1956, T-1957).

## Outcome
docs/specs/SPEC-0014-reference-array-results.md picks `RefGrid`, an array of references carried in
`RefArg::Grid` and `Binding::Grid` outside `CalcResult` (SPEC-0004's pattern), with a single value
view (`#VALUE!` per element). It lists the consumers by rule, the bounds and the rollout (T-1956,
T-1957, T-1838, new T-2063, T-2064, T-2065). Two new probes (probes/reference-lift-consumers and
-b1c, -b1d, -b2, 141 cells, excel-graph) answer the open questions: a top-left cell per element for
SUM/AVERAGE/MIN/MAX/PRODUCT, COUNT 0, per-element lifts for SUBTOTAL and the criteria functions,
`#VALUE!` per element to operators. Twelve statistical aggregates did not open in Excel for the web
(T-2065). Eval: probes +4 cases, 123 new-coverage known failures (the carrier is not built yet),
0 regressions.
