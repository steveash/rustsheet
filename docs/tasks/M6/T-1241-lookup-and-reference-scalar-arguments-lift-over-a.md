---
id: T-1241
title: "Lookup and reference scalar arguments lift over a range or an array (ADDRESS, INDIRECT, XLOOKUP, XMATCH, LOOKUP, OFFSET and more)"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The probed scalar arguments of ADDRESS, HYPERLINK, INDIRECT, LOOKUP, XLOOKUP, XMATCH, OFFSET, TRIMRANGE and FORMULATEXT lift over a range and an array, CHOOSECOLS/CHOOSEROWS read a range or array of indexes whole as a list, and vector/array arguments take array literals whole, as Excel does.

## Context
SPEC-0011 rollout step 3 (docs/specs/SPEC-0011-scalar-argument-lifting.md). Lift the lookup scalars whose elements are values through `lift_call`, with their parameters in `Function::arg_kinds` (`Lift`, or `Whole` for the vector and list arguments). Lifted elements that are references (OFFSET, INDIRECT) or whole arrays (TAKE, DROP) are outside the spec. Design them here, on top of `lift_values` and `cast::calc_result_to_array_node`, rather than as a new copy of the broadcast. CHOOSE's index lift (`choose_array`, lookup_and_reference/mod.rs) moves onto the driver here: `index_num` is `Lift`, and the value arms are `Whole` (SPEC-0011 "Today's lifting code").

T-1238's probe probes/lookup-argument-modes (excel-graph, 2026-10-09) shows the probed scalar arguments lift (CHOOSE already
does), CHOOSECOLS/CHOOSEROWS take a range or array of indexes whole as a list in any orientation (F7-F9: 3x2, 3x2,
2x2; do not lift them), and array literals are taken whole; see docs/excel-semantics/lookup.md "Lookup and reference function argument modes".
Failing rows (F/G/H cells): 1-4 ADDRESS, 7-10 CHOOSECOLS/CHOOSEROWS, 11-13 HYPERLINK, 14-15 INDIRECT, 16-17 LOOKUP, 20-21
OFFSET, 22-23 and 25 XLOOKUP, 26-27 and 29 XMATCH, 34-35 TRIMRANGE, 38 FORMULATEXT. Reuse the existing lift helpers (as the
T-1212/T-1218 lifts did). OFFSET and INDIRECT lifted over arrays return arrays of references; check how the engine
represents that before choosing a design (references.md). Split per function family if it outgrows size M.

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/2bb922eef72e785e Calc!A22 `=HYPERLINK("mailto:a@example.com",(DROP(Data!A5:A10,3,Data!A2)>0.2))` (friendly_name a 3x1 array): Excel `#SPILL!`, engine `#N/IMPL!`.
- gen-m7/cb748b6a9b81fbb7 Calc!A28 `=OFFSET(Data!A9:C10,1,VLOOKUP(..),3,OFFSET(Data!D6:D9,3,(Data!D$4+7)))` (width a 4x1 reference, cols `#N/A`): Excel `#SPILL!` (the lift wins over the `#N/A` scalar argument), engine `#N/A`.
- gen-m7/f68fafa0ca2eba26 Calc!A7 `=ADDRESS(15,..,3,((Data!$D$5*Data!B$12)>=FILTER(Data!D1:D12,..)))` (a1 an array): Excel `#SPILL!`, engine `#N/IMPL!`.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/675763979c1defb6 Calc!A25 `=UNICODE(INDIRECT("Data!A2:A4",VSTACK(Data!C6:E7)))` (a1 over an array): Excel `#SPILL!`, engine `#N/IMPL!`.
- gen-m9/adb3fd63c7368166 Calc!A4 `=LOOKUP(-OFFSET(Data!B2:B7,3,3),Data!B11:B11,Data!E7:E8)` (an array lookup_value): Excel `#SPILL!`, engine `#N/A`.
- gen-m9/adb3fd63c7368166 Calc!A34 `=INDIRECT(OFFSET(Data!A3:A5,3,1,3,3),(TANH(1)=Data!B12))` (ref_text over a 3x3 range): Excel `#SPILL!`, engine `#VALUE!`.

## Acceptance criteria
- [x] The listed probes/lookup-argument-modes cells pass (except those needing T-1242); baseline updated. Split: the
  reference-valued OFFSET, INDIRECT and TRIMRANGE rows (14-15, 20-21, 34-35) moved to T-1810.
- [x] gen-m7/2bb922eef72e785e A22 and gen-m7/f68fafa0ca2eba26 A7 pass; gen-m7/cb748b6a9b81fbb7 A28 (OFFSET) moved to T-1810.
- [x] gen-m9/adb3fd63c7368166 A4 passes, with no new failing gen-m9 cell and no new known failures in eval/baseline/;
  gen-m9/675763979c1defb6 A25 and adb3fd63c7368166 A34 (INDIRECT) moved to T-1810.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Read, not started; handed off by a sonnet run before any code (work-loop step 5b: a design question). Why:
- The task needs lifted elements that are references (OFFSET, INDIRECT return arrays of references, `#SPILL!` cells in gen-m7/gen-m9), which SPEC-0011 puts outside the driver ("Design them here, on top of `lift_values`"). That design is not settled and needs a decision on how the engine represents an array of references (references.md).
- Many functions with mixed `Lift`/`Whole`/`WholeByCaller` parameter kinds (CHOOSECOLS/CHOOSEROWS read a list whole; CHOOSE's `choose_array` moves onto the driver).
- Next: land T-1774 pattern first (merged, PR #1022: table entries in `functions/lift/kinds.rs`, `lifting_table_functions!` in the same file, `read_lift_arg` for single-argument reads); then split by family (value-only: ADDRESS, HYPERLINK, LOOKUP, XLOOKUP, XMATCH, FORMULATEXT, TRIMRANGE; reference-returning: OFFSET, INDIRECT as their own task after a design note).

## Outcome
Split on resumption (handoff above): this task lifted the value-returning lookup scalars, and T-1810 takes the
reference-valued lifts (OFFSET, INDIRECT, TRIMRANGE) and CHOOSE's move onto the driver.
- ADDRESS and HYPERLINK are lifting-table entries (`Lift` throughout) with `lift_call` kernels; their static
  signature and shape now come from the table.
- LOOKUP's lookup_value and XMATCH's lookup_value and match_mode lift through `lift_values`, with the searched
  vector (and LOOKUP's result_vector) read once for every element; their static shapes broadcast those arguments.
- CHOOSECOLS/CHOOSEROWS read each index argument whole as a row-major list (one shared `chosen_indices`, bounded
  before the result is built); FORMULATEXT over a multi-cell reference gives one result per cell, bounded in cells
  and text.
- Eval: probes known failures 4830 -> 4782 (48 lookup-argument-modes cells fixed), 0 regressions; gen-m7 A22, A7
  and gen-m9 A4 fixed, no other gen-suite cell changed.

