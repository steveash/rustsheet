---
id: T-3106
title: "Probe LOOKUP and XLOOKUP/XMATCH approximate and binary modes with a key a few ULP from a table number"
milestone: M6
lane: research
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for how LOOKUP (vector and array forms), XMATCH and XLOOKUP (match_mode -1/1, search_mode 1/-1/2/-2)
order a key a few ULP from a table number: exact doubles, as VLOOKUP/HLOOKUP/MATCH do (T-3073), or 15 digits.

## Context
- T-3099 made `SearchVector::approximate_search` and `descending_search` compare numbers exactly
  (`compare_searched`, functions/lookup_and_reference/lookup_table.rs). LOOKUP shares `approximate_search`, so it now
  compares exactly too, but no Excel cell covers LOOKUP with near-equal keys (docs/excel-semantics/lookup.md
  "Approximate match").
- XMATCH/XLOOKUP use `linear_scan` (xmatch.rs) and `binary_search` (functions/binary_search.rs), which still compare
  through `compare_values` (15 digits). Their exact mode uses 15-digit keys (`CollatedKey`); whether their
  approximate and binary modes are exact is unmeasured.
- Reuse the shapes of probes/t3073-lookup-near-equal-approximate (keys 1±k·2^-52 / 2^-53 against {2; X; 0.5},
  the run 1, 1+2^-52, 1+2·2^-52, 2, and 0.1+0.2 against 0.3) through the `excel-probe` skill.

## Acceptance criteria
- [ ] An oracle-filled probe case covers LOOKUP (both forms), XMATCH and XLOOKUP in every match_mode/search_mode
      above, with keys on both sides of 15-digit rounding.
- [ ] lookup.md records the result; any engine mismatch has a follow-up task (functions-lookup) with its cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine (follow-up task); criteria operators (T-3084).
