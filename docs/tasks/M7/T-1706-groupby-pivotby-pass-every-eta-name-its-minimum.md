---
id: T-1706
title: "GROUPBY/PIVOTBY: pass every eta name its minimum argument count (two-argument functions beyond the probed list)"
milestone: M7
lane: functions-lookup
status: done
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

## Outcome
`eta_arity` now looks the function up in `eta_min_args.rs`, a table of the catalog's non-1 minimum arguments, with a
unit test (`table_matches_catalog`) that keeps it in sync with `eval/catalog/excel-functions.tsv`. Probe
probes/groupby-eta-arity (excel-graph): COVARIANCE.S, SLOPE, SUMX2MY2, INTERCEPT get two arguments (grouped, shape 6x2),
RANK, PERCENTILE, MOD two (`#VALUE!` per cell, as an array result), SUBSTITUTE (minimum 3) makes the whole result
`#VALUE!`. Remaining probe failures are SLOPE/INTERCEPT size mismatches, T-1867.
