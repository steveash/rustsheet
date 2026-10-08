---
id: T-0235
title: "Aggregates: IFERROR blank is an empty value; HLOOKUP/VLOOKUP pass references; INDEX in intersection"
milestone: M2
lane: functions-logic
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The three T-0216 leftovers:
1. IFERROR/IFNA of a blank cell is an *empty value*. It is "" in `&` and FALSE/0 where a value is
   needed: `=IF(Data!E12,OR(IFERROR(Data!D9,Data!A9)))` with D9 blank → FALSE (today `#VALUE!`).
2. HLOOKUP/VLOOKUP results reach an aggregate as references: `=-COUNT(Data!E10:E12,IF(Data!$C3,Data!C5,Data!A4),HLOOKUP(Data!C$8,Data!A3:B4,Data!C9))`
   → -2, because the HLOOKUP picks B4 = FALSE, which COUNT skips (today -3).
3. `INDEX(A1:A2,2) A1:A2` intersects the reference INDEX returns (gives the A2 value). This was a T-0216
   acceptance item.
4. `ISREF(IF(TRUE,A1))` and `ISREF(CHOOSE(1,A1))` are TRUE (probes/isref-function-results C3, C5; known
   failures today). `ISREF(IF(TRUE,1))` and `ISREF(CHOOSE(1,5))` stay FALSE.

## Context
Evidence (excel-graph gen-m2 sidecars): 7469c3e920e1ade4 Calc!A351 (1), 60a543744fc4a8f7 Calc!A200 (2),
dd99be1f2454159f A110 and d0c29dd5c530e28c A112 (IFERROR blank is "" in `&`; these pass today, keep them).
T-0216 added `Model::evaluate_aggregate_arg` (functions/util.rs), plus `if_impl`/`index_impl` with an
`as_reference` flag. Extend it for VLOOKUP/HLOOKUP (they need the same flag). For (3), add an INDEX arm
to `evaluate_node_with_reference` in model.rs. model.rs is oversized, so offset the added lines elsewhere.
For (1), check how OR/AND and `&` treat `CalcResult::EmptyArg` vs `EmptyCell` before choosing what
IFERROR returns. Semantics: docs/excel-semantics/references.md.

## Acceptance criteria
- [ ] 7469c3e920e1ade4 A351 and 60a543744fc4a8f7 A200 leave eval/baseline/gen-m2.txt; no new F lines.
- [ ] Unit test for `=INDEX(A1:A2,2) A1:A2`.
- [ ] probes/isref-function-results C3 and C5 leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- fa437879253827be A372 (T-0218).
