---
id: T-3096
title: "Probe blank elements of a lifted VLOOKUP, HLOOKUP and MATCH lookup_value against tables holding 0, empty text, FALSE and blanks"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for what a blank cell inside a multi-cell lookup_value range looks up when VLOOKUP, HLOOKUP or MATCH
lift over it, exact and approximate, including the blank tail of a whole-column lookup_value, so that the engine
gives each such element Excel's value.

## Context
- docs/excel-semantics/lookup.md "VLOOKUP / HLOOKUP lift over an array lookup_value" ends "Unprobed: blank elements of
  a multi-cell lookup_value". The same section says a whole column lifts (`ROWS(VLOOKUP(D:D,A1:B4,2,FALSE))` is
  1048576), so every element below the used area is a blank element whose value is unprobed.
- Scalar evidence that brackets the question:
  - lookup.md "A reference-valued lookup_value pointing at a blank or error cell" (T-1371): a blank key gives `#N/A`
    against an all-number table with no 0, `""` or blank candidates, and says it does not test those candidates;
  - lookup.md "Approximate-match lookups: type-skipping binary search" records `MATCH(blank,{1,blank,0,blank,2})` = 3
    and `MATCH(blank,{"";0},0)` = 2: a blank key is the number 0;
  - lookup.md "MATCH lifts over lookup_value" (T-1861): `MATCH(r&"",r&"",)` turns a blank into `""`, but the bare
    range elements are not probed;
  - lookup.md "Whole-column / whole-row search vectors: blank tail": a blank-cell key in `VLOOKUP(G1,A:B,2,FALSE)` is
    `#N/A` over a column with no 0.
  So a lifted blank element might be the number 0 (and find a 0 row), the empty value (and find nothing), or `""`.
- Engine: `Model::lookup_each` (functions/lookup_and_reference/lookup_each.rs) lifts through `lift_values` with
  `REUSE_EQUAL_RUNS` and hands each element to `lookup_index`; `Model::fn_match` lifts the same way. What a blank
  element becomes there is not pinned by any Excel cell.
- Why opus: the cases must tell 0, `""`, FALSE and "nothing" apart in exact and approximate searches (approximate
  binary search over a table holding 0 lands differently for a 0 key than for a skipped blank, lookup.md
  "Approximate-match lookups"), and the whole-column rows must stay within the cell-visit cap (criteria.md) while
  reading tail elements through `INDEX(..,1000)` rather than spilling a million cells.

## Acceptance criteria
- [x] A new excel-graph probe case `probes/t3096-lookup-value-blank-elements` (milestone M2, decision 0043; not M1)
      reads back, for lookup_value ranges whose middle cell is blank (one column for VLOOKUP and MATCH, one row for
      HLOOKUP): `VLOOKUP(range,table,2,FALSE)`, `VLOOKUP(range,table,2,TRUE)`, `HLOOKUP` both ways and `MATCH` with
      match_type 0, 1 and -1, each against four tables (keys with a 0 row, with a `=""` row, with a FALSE row, and with
      only a blank row among numbers), every element read through `INDEX(..,n)`; the same through an array lookup_value
      holding a blank element (`IF({1;0;1},range,blankcell)`); a whole-column lookup_value (`VLOOKUP(D:D,..)`,
      `MATCH(D:D,..,0)`) read at a row inside and a row far below the used area; and the scalar blank-cell key against
      the same four tables as the control row.
- [x] lookup.md replaces "Unprobed: blank elements of a multi-cell lookup_value" with the rule and its cells, and the
      T-1371 section's "does not test those candidates" sentence cites the scalar control rows.
- [x] Every cell the engine does not match is a known failure in eval/baseline/probes.txt, and one follow-up task
      (milestone M2, lane functions-lookup, tier by decision 0018) names those cells; `make exit-owners MILESTONE=M2`
      reports 0.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the follow-up owns them).
- XLOOKUP, XMATCH and LOOKUP (M6): file a probe task in M6 if a lifted blank element is not the scalar blank key.
- Blank elements of the index or range_lookup arguments (T-2234 probed blank index elements).

## Outcome
- probes/t3096-lookup-value-blank-elements (200 formula cells, excel-graph 2026-10-29): a blank element of a lifted
  VLOOKUP, HLOOKUP or MATCH lookup_value (range middle cell, IF-array element, whole column inside and at row 1000) is
  the scalar blank key, the number 0, in every exact and approximate form against tables holding 0, `""`, FALSE and a
  blank row; it is never `""` or FALSE and never matches a blank row.
- lookup.md: the "Unprobed" sentence of "VLOOKUP / HLOOKUP lift over an array lookup_value" is replaced by the rule;
  the T-1371 section cites the scalar control rows.
- Eval delta: +1 case, 203/203 cells pass (engine already matched); no known failures, so no follow-up task, and no
  M6 XLOOKUP probe task is needed (the out-of-scope condition, a lifted blank differing from the scalar blank key, does
  not hold).
