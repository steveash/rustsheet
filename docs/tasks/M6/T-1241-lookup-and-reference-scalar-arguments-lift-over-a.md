---
id: T-1241
title: "Lookup and reference scalar arguments lift over a range or an array (ADDRESS, INDIRECT, XLOOKUP, XMATCH, LOOKUP, OFFSET and more)"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: M
tier: sonnet
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
- [ ] The listed probes/lookup-argument-modes cells pass (except those needing T-1242); baseline updated.
- [ ] gen-m7/2bb922eef72e785e A22, gen-m7/cb748b6a9b81fbb7 A28 and gen-m7/f68fafa0ca2eba26 A7 pass. (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] gen-m9/675763979c1defb6 A25 and adb3fd63c7368166 A4 and A34 pass, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
