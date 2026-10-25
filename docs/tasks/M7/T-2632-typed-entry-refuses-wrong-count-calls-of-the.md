---
id: T-2632
title: "Typed entry refuses wrong-count calls of the functions outside the lifting table (=SUM(), =IF(1))"
milestone: M7
lane: engine
status: done
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
- [x] `=SUM()`, `=IF(1)` and a too-long XLOOKUP fail typed with `EditError::InvalidInput` and stay accepted stored; every
  built-in's count range is tested against excel-functions.tsv's minimum.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Progress notes (2026-10-25)
Claimed at the end of a session's budget and left unstarted for the next opus run (routine step 2, after 2 h idle). Starting
points found: `Parser::note_argument_count` (parser/operands.rs) asks only `Function::arg_kinds`; the ~300 other functions
take their signature from `fallback_signature` (parser/static_analysis.rs). A catalog-checked minimum table already exists:
`functions/lookup_and_reference/eta_min_args.rs` + `eta_min_args.txt` (names whose minimum is not 1, checked by
`group_functions.rs` `table_matches_catalog`); extend or move it (one definition per concept) rather than writing a second table.
Maximums are not in the catalog's min_args column; they would come from its `syntax` column (`...` = repeating) and need care
for engine-only extras (ACCRINTM's 3 arguments).

Resumed 2026-10-25 07:50 UTC by an opus run (routine step 2).

## Outcome
- Probed typed (Graph PATCH, 2026-10-25): `=SUM()`, `=IF(1)`, `=ABS(1,2)`, three-argument ACCRINTM and the off-cycle
  `=SORTBY(C1:C2,D1:D2,1,E1:E2)` are all refused (400 `InvalidArgument`); `=ABS(1)` is accepted. Recorded in syntax.md.
- `functions/lookup_and_reference/eta_min_args.{rs,txt}` became `functions/kinds/arg_counts.{rs,txt}`: one `NAME MIN MAX`
  line per catalog function (527). MIN is excel-functions.tsv's; MAX is Microsoft's syntax line (255 when it repeats or does
  not parse), widened by a non-repeating functions.yaml signature and a cited override (INDEX's reference form, 4). The
  functions.yaml repeat caps are generator bounds, so they never narrow it (review round 1: they refused 4-argument INDEX
  and GETPIVOTDATA with three pairs). `table_matches_catalog` (Rust) pins the minimums and
  `test_engine_arg_counts_match_the_catalog` (tools/tests) the whole table. GROUPBY/PIVOTBY's eta arity reads it.
- `Parser::note_argument_count` refuses typed by the catalog's counts for every function, plus whole `repeat` cycles for
  lifted functions (`ArgKinds::accepts`; was `takes_count`). ACCRINTM with three arguments is now refused typed.
- Eval: no change (no `entered` case has a wrong-count call). API test extended.
- Left open (UNVERIFIED in syntax.md): off-cycle counts of repeating functions outside the lifting table (IFS, SWITCH, LET)
  stay accepted typed. Probing typed maxima (max and max+1) is T-2682.
