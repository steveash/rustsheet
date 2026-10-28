---
id: T-3073
title: "Probe whether approximate VLOOKUP, HLOOKUP and MATCH compare numbers at 15 digits or exactly (exact match is exact)"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for whether the approximate searches of VLOOKUP/HLOOKUP (range_lookup TRUE) and MATCH (match_type 1
and -1) treat a table number a few ULP from the lookup value as equal to it (15-digit values) or as larger/smaller
(exact doubles), including how far the "run of equal values" extends, so the engine's search comparator is backed by
cells.

## Context
- docs/excel-semantics/lookup.md "Approximate-match lookups: type-skipping binary search": exact match needs exactly
  equal numbers (`MATCH(0.1+0.2,{0.3},0)` and `VLOOKUP(0.1+0.2,{0.3},1,FALSE)` are `#N/A`,
  probes/lookup-approximate-search N37, N38), but no probe puts a near-equal number in an approximate search:
  lookup-approximate-search and tiny-vs-zero-criteria (MATCH/VLOOKUP TRUE against 1E-300, 1E-17, 2E-16 and 0, where
  15-digit and exact compare agree) are the only numeric evidence.
- The engine compares with `compare_values` (functions/util.rs, `cmp_excel_precision`: 15-digit values) in
  `SearchVector::approximate_search`, its equal-run walk and `SearchVector::descending_search`
  (functions/lookup_and_reference/lookup_table.rs), while `exact_position` compares raw doubles. So with A1 =
  `=0.1+0.2` (0.30000000000000004, not a near-zero root), `MATCH(0.3,A1,1)` is 1 in the engine and would be `#N/A`
  under an exact compare; over `1`, `=1+2^-52`, `2` the key 1 stops at position 2 (the run's end) under 15 digits
  and at 1 exactly. Either way the wrong reading is visibly off (a different row or `#N/A`).
- Opus: the 15-digit cut is not a relative epsilon (numbers.md "Comparison tolerance": k = 3-8 ULP is equal at some
  bases only), so the probe needs ULP ladders on both sides of the key and a pair straddling a 15th-digit rounding
  boundary, and the follow-up changes a search shared by every lookup consumer (LOOKUP, XLOOKUP and XMATCH reuse
  `SearchVector`, M6).

## Acceptance criteria
- [ ] A new excel-graph probe case `probes/t3073-lookup-near-equal-approximate` (milestone M2, decision 0043; not M1)
      reads back, with table numbers built by formula (literals are cut to 15 digits): `VLOOKUP(key,table,2,TRUE)`,
      `HLOOKUP(key,table,2,TRUE)`, `MATCH(key,col,1)` and `MATCH(key,col,-1)` (descending tables) for key 1 against
      `=1+k*2^-52` and `=1-k*2^-53` with k = 1, 2, 3, 8; the same with the near value as the key and 1 in the table;
      an equal run (`1`, `=1+2^-52`, `=1+2*2^-52`, `2`) to locate where the run ends; the `=0.1+0.2` against 0.3 pair
      in both directions; and a pair a few ULP apart on either side of a 15th-digit rounding boundary. Each row
      returns an index or a tag column so the position is read, not only the value.
- [ ] docs/excel-semantics/lookup.md "Approximate-match lookups" and "MATCH descending search" record whether the
      searches compare at 15 digits or exactly, and where the equal run ends, citing the cells.
- [ ] Every cell the engine does not match is a known failure in eval/baseline/probes.txt, and one follow-up task
      (milestone M2, lane functions-lookup, tier opus) names those cells and the lookup_table.rs functions to change;
      `make exit-owners MILESTONE=M2` reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the follow-up task owns them).
- LOOKUP, XLOOKUP and XMATCH approximate and binary modes (M6): note in the follow-up whether they share the answer,
  and file an M6 probe task if they need their own cells.
- Criteria ordering operators (T-3072). Text and boolean keys.
