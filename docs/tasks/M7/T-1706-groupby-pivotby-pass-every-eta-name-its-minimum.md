---
id: T-1706
title: "GROUPBY/PIVOTBY: pass every eta name its minimum argument count (two-argument functions beyond the probed list)"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
GROUPBY and PIVOTBY call every function passed by name with its minimum argument count, as Excel does: one for a
minimum of 1, two (the values and the relative set) for a minimum of 2, and the whole result `#VALUE!` for 0 or 3+.

## Context
T-1325 found the rule (probes/groupby-pivotby-functions G!A421, -functions2 G!A281-A501: PERCENTOF, CORREL, PEARSON,
RSQ, COVARIANCE.P, IF, ROUND and SUMIF get two arguments; NOW, TEXTJOIN and XLOOKUP make the result `#VALUE!`;
SUMPRODUCT, MEDIAN, ABS get one) but `eta_arity` in
crates/rustsheet-engine/src/functions/lookup_and_reference/group_functions.rs only lists the probed functions; every
other function gets one argument. The minimum arities are in eval/catalog/excel-functions.tsv (column 3), which the
engine cannot read at run time: generate a table (or a checked `match`) from it with a hygiene check that it stays in
sync, and replace the list. Probe a few unlisted two-argument functions (e.g. COVARIANCE.S, SLOPE, SUMX2MY2) and a
three-argument one (e.g. SUBSTITUTE) to confirm. Docs: docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY".

## Acceptance criteria
- [ ] `eta_arity` covers every implemented function from the catalog's minimum arity; the TODO(T-1706) is gone.
- [ ] A probe of unlisted two- and three-argument functions passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
