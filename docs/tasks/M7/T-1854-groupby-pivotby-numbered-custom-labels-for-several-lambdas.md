---
id: T-1854
title: "GROUPBY/PIVOTBY numbered CUSTOM labels for several LAMBDAs"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The function labels of a GROUPBY or PIVOTBY function array with two or more LAMBDAs are Excel's
("CUSTOM1", "CUSTOM2", ...), not "CUSTOM" for each.

## Context
Found by T-1700. With `HSTACK(LAMBDA(x,ARRAYTOTEXT(x)),LAMBDA(x,"n"&COUNT(x)))` (as columns) and the
`VSTACK` of the same two (as rows), Excel labels the functions "CUSTOM1" and "CUSTOM2"
(probes/pivotby-col-subtotals-values2 S!A181 row 183, S!A193 column B; probes/pivotby-col-subtotals-values3
S!A1-A25 row 3); the engine prints "CUSTOM" for both. One LAMBDA beside a named function is "CUSTOM"
(probes/groupby-pivotby-functions G!A181, G!A721). These 130 cells are the only known failures of those two
cases. docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY" (Labels) records it. Code: the label is built in
functions/lookup_and_reference/group_functions.rs (`"CUSTOM"`).

## Acceptance criteria
- [ ] Probe first (excel-probe skill): three LAMBDAs, LAMBDA/SUM/LAMBDA (is the numbering per LAMBDA or per
  position?), one LAMBDA alone in HSTACK with a label, the same in GROUPBY; record the rule in arrays.md.
- [ ] The cells above and the new probe cells pass; `make check` green; reviewers approve.

## Out of scope
- Labels of named functions.

## Outcome
GROUPBY/PIVOTBY number the default labels of two or more LAMBDAs ("CUSTOM1", "CUSTOM2", ...) per LAMBDA in order;
a single LAMBDA stays "CUSTOM". Probe `probes/groupby-custom-labels` (excel-graph) records the rule in
`docs/excel-semantics/arrays.md`. Eval: probes known failures 2214 -> 2084 (130 fixed, 0 regressions).
Unprobed: whether a labelled LAMBDA counts in the numbering (`HSTACK(L,L,"lab")` was undecodable).
