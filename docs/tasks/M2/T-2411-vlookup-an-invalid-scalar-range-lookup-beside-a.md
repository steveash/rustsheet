---
id: T-2411
title: "VLOOKUP: an invalid scalar range_lookup beside a lifted lookup_value is #VALUE! per element, not one #VALUE! (gen-m7)"
milestone: M2
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
VLOOKUP (and HLOOKUP, which shares the path) with a lifted lookup_value and a scalar range_lookup that is invalid (text such as "1")
returns an array of that `#VALUE!`, one per lookup_value element, as each element's scalar call would, instead of one `#VALUE!`.

## Context
Gap triage of the T-2254 gen-m7 sweep (excel-graph, 2026-10-22); 8 cells, 1 formula.
`WRAPROWS(Data!E6:E11,1,VLOOKUP(OFFSET(Data!D6:E8,2,0),Data!B5:C5,3,IF(Data!$D8,"1")))` is a 3x2 grid of 3.14159 (f7e94a817e5385ac
A2430:B2432; nothing below). WRAPROWS's pad_with lifts (arrays.md "EXPAND, WRAPROWS and WRAPCOLS: scalar arguments lift"), each element
being the top-left of `WRAPROWS(Data!E6:E11,1,<element>)`, i.e. E6 = 3.14159; so Excel's VLOOKUP there is a 3x2 array (the lifted
lookup_value `OFFSET(Data!D6:E8,2,0)` is 3x2), whatever its elements. The range_lookup is the text "1" (D8 is TRUE). Ours: the VLOOKUP is
one `#VALUE!` (`VLOOKUP({1,2},Data!B5:C5,3,"1")` is one `#VALUE!` too, checked 2026-10-22), so WRAPROWS does not lift and spills E6:E11.
lookup.md "VLOOKUP / HLOOKUP with an array index or range_lookup" already says lookup_value and range_lookup lift together and each pair is
a scalar call, "so an error or text range_lookup element is that element's result" (`{"a";"b"}` is `#VALUE!` twice); a scalar text
range_lookup beside an array lookup_value is the same broadcast. Code: the VLOOKUP/HLOOKUP range_lookup reading in
functions/lookup_and_reference/ (validated once before the lift today). Quirk entry: lookup.md, sentence added by this triage
(UNVERIFIED for the element values themselves, which the cell cannot show: `#VALUE!` per element follows from the scalar rule; confirm with
`INDEX(VLOOKUP({1;2},A1:B2,2,"1"),2)` and `ROWS(..)` in a probe cell if one is cheap to add).

Owned gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22): 8 known-failure lines in eval/baseline/gen-m7.txt, 1 formulas.
- gen-m7/f7e94a817e5385ac.case.json A2430 `=WRAPROWS(Data!E6:E11,1,VLOOKUP(OFFSET(Data!D6:E8,2,0),Data!B5:C5,3,IF(Data!$D8,"1")))`: B2430 Excel 3.14159, ours <empty>; cells (8): B2430, A2431:B2431, A2432:B2432, A2433, A2434, A2435

## Acceptance criteria
- [x] Visible parity: these 8 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new
      known-failure lines in any baseline.
- [x] Unit tests: `ROWS(VLOOKUP({1;2},A1:B2,2,"1"))` is 2 with `#VALUE!` elements; the scalar `VLOOKUP(1,A1:B2,2,"1")` stays one `#VALUE!`.
- [x] lookup.md's entry has an "Ours:" line naming this task.
- [x] `make check` green; reviewers approve.

## Out of scope
- An array index (its own rule in lookup.md).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
`lookup_head` (functions/lookup_and_reference/index_arg.rs) now keeps a scalar `range_lookup`'s coercion error when
`lookup_value` lifts. Each element's scalar call then gives that error, and an error element of `lookup_value`
still wins. A scalar call is still one `#VALUE!`. This covers VLOOKUP and HLOOKUP, which share the path.
Element values were probed in probes/vlookup-scalar-range-lookup-lifted (32 cells, excel-graph, 2026-10-23). The probe
found that a failed value index (text or error) in any lifted call (a lifted lookup_value, an array range_lookup, or
both) is the index error in every element. It beats a bad range_lookup and a not-found element, and only an error
lookup_value element keeps its own error. Before this task, ours gave one error beside a scalar range_lookup, and `#N/A`
for a not-found element beside an array range_lookup (`LookupIndexArg::Failed`). Both cases now go through the
table-error path (`argument_error_each`), and `Failed` is removed.
Tests: `bad_scalar_range_lookup_beside_a_lifted_lookup_value_is_an_error_per_element` and
`bad_value_index_beside_a_lifted_lookup_value_is_an_error_per_element`. lookup.md's entry is rewritten from the probe.
Eval delta: gen-m7 known failures 706 -> 698 (the 8 owned lines), 0 regressions; probes +1 case (32 cells, all pass).
