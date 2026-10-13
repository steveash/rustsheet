---
id: T-1468
title: "VLOOKUP/HLOOKUP lift over a multi-cell range range_lookup (4 gen-m2 known failures)"
milestone: M2
lane: functions-lookup
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
A VLOOKUP or HLOOKUP whose range_lookup argument is a multi-cell *reference* (a range, or INDEX/HLOOKUP
giving a range or an array) lifts over it like an array range_lookup and spills, as Excel does. Today
the engine intersects it implicitly and gives `#VALUE!`.

## Context
docs/excel-semantics/lookup.md "VLOOKUP / HLOOKUP with an array index or range_lookup" verifies the lift
for an *array* range_lookup (T-1269). It lists as "Unprobed" a valid multi-cell *range* range_lookup with
a scalar lookup_value, and notes that the engine intersects it implicitly while static analysis marks
the formula as spilling. gen-m2 (excel-graph, T-0203) has the evidence. Excel lifts, and the result spills:
- `gen-m2/942bdade381b7537` Calc!A381 `VLOOKUP(..,Data!A3:A9,..,INDEX(Data!A2:A3,0.5))&"ü"`: INDEX with
  row 0.5 (truncated to 0) is the 2x1 range A2:A3. Excel gives `#SPILL!` (A382 is occupied); ours is `#VALUE!`.
- `gen-m2/d3e2df0bef2f173a` Calc!A20 `VLOOKUP(CONCATENATE(Data!C10),Data!B7:E8,1,INDEX(Data!A8:B12,0,0))*0.3`:
  Excel `#SPILL!`, ours `#VALUE!`.
- `gen-m2/f7e6d351c8538ef5` Calc!A296 `VLOOKUP(..,HLOOKUP(..,INDEX(Data!A8:C9,Data!B2,3)))`: B2 is 0, so
  the INDEX is the 2x1 range C8:C9. The HLOOKUP lifts, and so does the outer VLOOKUP. Excel `#SPILL!`, ours `#VALUE!`.
- `gen-m2/04cfd82f54dbc2b2` Calc!A19 `(..)=VLOOKUP((Data!A$3<=7),Data!A11:A12,1,INDEX(Data!B4:E9,2,0.1))`:
  the INDEX is the 1x4 row B5:E5. The spill fits, and its first element is `#N/A` (an exact search for
  FALSE). Excel `#N/A`, ours `#VALUE!`.
Run `./target/release/rustsheet-eval show gen-m2/<case>.case.json` for the Data sheet. The lift itself
(broadcast with lookup_value, each element a scalar call, the index taking its first element when
range_lookup lifts) is already implemented for arrays in `Model::lookup_each`
(functions/lookup_and_reference/lookup_each.rs). Do not change the index argument. A multi-cell range
*index* stays implicitly intersected (lookup.md, probes/lookup-index-order, VERIFIED).

## Acceptance criteria
- [ ] The 4 cells above leave eval/baseline/gen-m2.txt via `make eval-baseline`, with no new F line in any suite.
- [ ] lookup.md: the "Unprobed" note on a multi-cell range range_lookup is replaced by the rule, citing these 4 cells.
- [ ] A unit test covers a range range_lookup (one spilling cell, one with a `#SPILL!` collision).
- [ ] `make check` green; reviewers approve.

## Out of scope
- A multi-cell range index (stays intersected; T-1426 probes the IF-branch form).
- Error precedence between a lookup_value element and a range_lookup element (unprobed; file a probe task in M2 if needed).

## Outcome
A multi-cell range range_lookup now lifts like an array (`lookup_sorted_arg`), and an error lookup_value or index no longer short-circuits when range_lookup lifts (`lookup_head`). All 4 gen-m2 cells leave the baseline (gen-m2 known failures 10 -> 6). Which error wins between several failing arguments stays unprobed.
