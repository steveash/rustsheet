---
id: T-3099
title: "Approximate VLOOKUP, HLOOKUP and MATCH compare numbers exactly, not at 15 digits"
milestone: M2
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The approximate searches of VLOOKUP/HLOOKUP (range_lookup TRUE) and MATCH (match_type 1 and -1) compare numbers as
exact doubles, as Excel does, so a table number a few ULP from the key is larger or smaller than it, never equal.

## Context
- T-3073 measured it (docs/excel-semantics/lookup.md "Approximate-match lookups", "MATCH descending search"):
  every cell of probes/t3073-lookup-near-equal-approximate is an exact compare, none a 15-digit one.
  `MATCH(0.3,{2;0.1+0.2;0.5},1)` is `#N/A`, and over 1, 1+2^-52, 1+2*2^-52, 2 key 1 is position 1, not the run's end 3.
- The engine fails 69 lookup cells of t3073-lookup-near-equal-approximate, all of them in Sheet1: A1-A7, A9-A15, A17, A30, A31 and the same rows of B and C
  (VLOOKUP, HLOOKUP, MATCH 1), and D1-D7, D9-D15, D17, D18, D30, D31 (MATCH -1). Sheet1!E8 is a near-zero snap and belongs to T-3100.
- Code: `SearchVector::approximate_search`, its equal-run walk and `SearchVector::descending_search`
  (functions/lookup_and_reference/lookup_table.rs) call `compare_values` (functions/util.rs), which compares
  numbers with `cmp_excel_precision` (15 digits). `exact_position` already needs exact equality for numbers.
- LOOKUP, XLOOKUP and XMATCH reuse `SearchVector` (M6). Check whether the change moves any of their cells in gen-m6
  or the probes. If their approximate or binary modes need cells of their own, file an M6 probe task.

## Acceptance criteria
- [x] Every lookup cell of probes/t3073-lookup-near-equal-approximate listed above passes, `make eval-baseline` is run, and there are no regressions.
- [x] lookup.md's "Implemented in" names the exact compare.
- [x] `make check` green; reviewers approve.

## Out of scope
- Text and boolean keys; criteria operators (T-3084).

## Outcome
`SearchVector::approximate_search` (and its equal-run walk) and `SearchVector::descending_search` now order numbers
through a new `compare_searched` (lookup_table.rs): exact doubles for number pairs, `compare_values` otherwise. All 69
lookup cells of probes/t3073-lookup-near-equal-approximate pass (eval/baseline/probes.txt -69 lines, no regressions,
no other cell of any suite moved, so LOOKUP's shared search, gen-m2 and gen-m6 are unchanged). Sheet1!E8 stays with
T-3100. Unit test `approximate_searches_compare_numbers_as_exact_doubles`; lookup.md names the exact compare.
