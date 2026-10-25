---
id: T-2632
title: "Typed entry refuses wrong-count calls of the functions outside the lifting table (=SUM(), =IF(1))"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Typed entry refuses a call with fewer or more arguments than the function takes for every built-in, not only the lifting table's.

## Context
Found by T-2568: `Parser::note_argument_count` (parser/operands.rs) checks only functions with `Function::arg_kinds`; the
others (SUM, IF, LET, INDEX, XLOOKUP, ...; `fallback_signature` in static_analysis.rs) are never refused typed, and evaluate to
`#ERROR!`. Excel refuses such formulas in a file (docs/excel-semantics/syntax.md "Minimum argument counts"; `=IF(TRUE)`) and
typed (`=PV(1)`, T-2567). The counts must come from one place: give the fallback functions a count table (or reuse
eval/catalog/excel-functions.tsv's minimums through a generated table) checked against excel-functions.tsv by a test, rather
than trusting the hand-written signatures. Watch REDUCE/SCAN (three positions needed) and repeating pairs (IFS, SWITCH).
Probe (excel-probe skill, typed Graph PATCH) the cases T-2568 inferred or left open: a typed call with too many arguments
(`=ABS(1,2)`), an off-cycle SORTBY (`=SORTBY(A1:A2,B1:B2,1,C1:C2)`), and ACCRINTM with three arguments. TEXTJOIN's lifting-table maximum is 255 where Microsoft documents 252 texts (254
arguments). Also decide ACCRINTM's `par`: the engine takes 3 arguments, Microsoft's syntax line says 4 (excel-functions.tsv).

## Acceptance criteria
- [ ] `=SUM()`, `=IF(1)` and a too-long XLOOKUP fail typed with `EditError::InvalidInput` and stay accepted stored; every
  built-in's count range is tested against excel-functions.tsv's minimum.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
