---
id: T-2456
title: "VLOOKUP: probe an error lookup_value element beside a failed index, and each element of the T-2411 two-element claims"
milestone: M2
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Every claim T-2411 made in lookup.md ("VLOOKUP / HLOOKUP with an array index or range_lookup") is backed by one probe
cell per element, including the precedence of an error lookup_value element over a failed value index.

## Context
Minor findings of the T-2411 Excel-fidelity review (2026-10-23). The unit test
`bad_value_index_beside_a_lifted_lookup_value_is_an_error_per_element` (crates/rustsheet-engine/src/test/lookup_and_reference/test_fn_vlookup.rs)
asserts `VLOOKUP({1;#N/A},A1:B2,"a")` is `#VALUE!`,`#N/A`; no probe cell reads that second element (the rule is inferred from
probes/lookup-argument-edges H120-H124 and probes/vlookup-scalar-range-lookup-lifted G5). lookup.md also says HLOOKUP gives
"2 columns of `#VALUE!`" (G12 reads only column 2) and a `"TRUE"` range_lookup gives "one","two" (G19 reads only the second).

## Acceptance criteria
- [ ] A probe case (excel-probe skill) reads `ERROR.TYPE(INDEX(VLOOKUP({1;#N/A},A1:B2,"a"),2))` and each element of the
      HLOOKUP and `"TRUE"` cases above through INDEX, with oracle values.
- [ ] Ours matches every new cell, or a follow-up task names each mismatch; lookup.md cites the new cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing VLOOKUP behavior beyond what a mismatching cell shows.
