---
id: T-0216
title: "IF and INDEX return references, IFERROR returns values, in aggregate arguments (8 gen-m2 known failures)"
milestone: M2
lane: functions-logic
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
When IF (and INDEX) select a cell reference, aggregates (SUM, AVERAGE, COUNT, MIN, MAX, ...) treat the
result as a **reference**: text and blank cells are skipped, as for a direct range argument. IFERROR
returns a **value** (a blank cell becomes 0, text stays text).

## Context
gen-m2 (T-0203): 8 failing cells. Evidence (excel-graph):
- `gen-m2/96a0952412cefa6c` Calc!A88 `=ABS(SUM(IF(FALSE,(-2.5),Data!E10),Data!D4:D7))` (E10 = "abc"): expected 123456799.623, actual `#VALUE!`.
- `gen-m2/1b6ceed7489e0800` Calc!A206 `=(COUNT(Data!B1:B4,IF(Data!C6,Data!C$8,Data!$B12))/...)` (C8 = "1" text, not counted): expected -30000, actual -40000.
- `gen-m2/942bdade381b7537` Calc!A212 `=SUM(Data!E7:E7,INDEX(Data!B11:C11,1))` (B11 = " x "): expected 0.1, actual `#VALUE!`.
- IFERROR is a value: `gen-m2/caa981b0753f455c` Calc!A300 `...AVERAGE(IFERROR(Data!E7,Data!D7),IF(Data!A3,Data!E10),Data!C8:C9)`
  (E7 is 0, not blank; IF to "x" is skipped): expected -1.4999999999998498e-15, actual `#VALUE!`.
  `gen-m2/7469c3e920e1ade4` Calc!A351 `=IF(Data!E12,OR(IFERROR(Data!D9,Data!A9)))` (D9 blank gives 0, so FALSE): expected FALSE, actual `#VALUE!`.
- Other cells: `gen-m2/60a543744fc4a8f7` Calc!A200, `gen-m2/f0d1bacec5de8146` Calc!A183,
  `gen-m2/fa437879253827be` Calc!A372 (the last also needs T-0218).

Code: `crates/rustsheet-engine/src/functions/logical/mod.rs` (`fn_if`, `fn_iferror`), INDEX in
`lookup_and_reference/mod.rs`, and the shared aggregate-argument iterator from T-0200 (reuse it, do not
add a second classification).

## Acceptance criteria
- [ ] `=INDEX(A1:A2,2) A1:A2` intersects the returned reference -> moved to T-0233.
- [x] `ISREF(INDEX(A1:A2,2))` is TRUE.
- [x] gen-m2 96a0952412cefa6c A88, 1b6ceed7489e0800 A206, 942bdade381b7537 A212, caa981b0753f455c A300,
      f0d1bacec5de8146 A183 leave eval/baseline/gen-m2.txt with no new F lines. 7469c3e920e1ade4 A351 and
      60a543744fc4a8f7 A200 -> T-0233; fa437879253827be A372 -> T-0218.
- [x] docs/excel-semantics/references.md entry with the evidence (IF/INDEX reference, IFERROR value).
- [x] `make check` green; reviewers approve.

## Out of scope
- MIN/MAX on text values (T-0217).

## Outcome
There is now one aggregate-argument evaluator, `Model::evaluate_aggregate_arg` (functions/util.rs). It
returns a `Range` for a cell reference, and for an IF (scalar condition) or INDEX that selects one, through
`if_impl`/`index_impl` with `as_reference`. Everything else stays a value, so VLOOKUP/arithmetic/`&`
consumers are unchanged. SUM, COUNT, MIN, MAX and the AVERAGE family (`for_each_value`) use it; SUM's
ad-hoc ReferenceKind→RangeKind rewrite is gone. `ISREF(INDEX(…))` is TRUE unless the INDEX fails
(`ISREF(INDEX(A1,1,2,2))` stays FALSE, ironcalc ISREF_ISFORMULA!F9).
Eval (on main after T-0213): gen-m2 880 -> 875, 5 fixed (96a0952412cefa6c A88, 1b6ceed7489e0800 A206, 942bdade381b7537 A212,
f0d1bacec5de8146 A183, caa981b0753f455c A300), 0 regressions. Not done here:
7469c3e920e1ade4 A351 (IFERROR blank as an empty value), 60a543744fc4a8f7 A200 (HLOOKUP passes a
reference) and the INDEX intersection item. They are follow-up T-0233; fa437879253827be A372 waits on T-0218.

## Findings (opus run, 2026-10-05)
A first attempt was reverted because it regressed 21 gen-m2 cells and fixed only 7. It
made `fn_if`'s scalar branches use `evaluate_node_with_reference`, made INDEX return a 1x1 `Range` instead
of the cell value, added Index to `Function::returns_reference`, and turned IFERROR's blank into 0.
- IF/INDEX returning a reference **everywhere** is wrong. Value consumers then see a 1x1 range:
  `VLOOKUP(IF(TRUE,Data!A9),…)` gives #N/A (207a5753da48c7e0 A270), `x+IF(TRUE,Data!D11)`
  (b4392a6514a36ade A115), and `COUNT((C5+2)-IF(D5,"",D1))` (61cacc22c4244179 A135) regress. The reference
  must be visible only where Excel takes references: aggregate arguments, ISREF and the intersection operator.
- IFERROR of a blank is an empty *value*: "" in `&` (dd99be1f2454159f A110, d0c29dd5c530e28c A112).
  Do not map it to 0. The task's A300 note was wrong: E7 there is 0.
- There is no shared aggregate-argument classifier. SUM (`fn_sum`), COUNT (`fn_count`), the AVERAGE family
  (`for_each_value`) and MIN/MAX each walk args themselves and test `matches!(arg, Node::ReferenceKind)`.
  Suggested design: one `Model::evaluate_aggregate_arg(node, cell)`. It returns a `Range` for a reference
  node, and for IF/CHOOSE/INDEX whose selected result is a reference (recursively); otherwise it returns the
  value. Route every aggregate through it (T-0200's intent), then T-0217 can land on top.

