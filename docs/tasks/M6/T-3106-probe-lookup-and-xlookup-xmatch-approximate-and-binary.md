---
id: T-3106
title: "Probe LOOKUP and XLOOKUP/XMATCH approximate and binary modes with a key a few ULP from a table number"
milestone: M6
lane: research
status: done
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
- [x] An oracle-filled probe case covers LOOKUP (both forms), XMATCH and XLOOKUP in every match_mode/search_mode
      above, with keys on both sides of 15-digit rounding.
- [x] lookup.md records the result; any engine mismatch has a follow-up task (functions-lookup) with its cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine (follow-up task); criteria operators (T-3084).

## Outcome
- probes/t3106-lookup-xmatch-near-equal (M6, excel-graph 2026-10-29): the 20 key/X pairs of T-3073 and its two runs,
  through LOOKUP (vector, tall and wide array forms, a sorted vector), XMATCH and XLOOKUP in match_mode 0/-1/1 under
  search_mode 1/-1 (unsorted {2;X;0.5}), 2 ({0.5;X;2}) and -2 ({2;X;0.5}).
- Finding: every function and every mode compares numbers as exact doubles, XMATCH/XLOOKUP's exact mode included.
  Recorded in lookup.md "XMATCH, XLOOKUP and LOOKUP compare numbers as exact doubles"; the approximate-match section
  no longer calls LOOKUP unverified. AC8 repeats T-3073's `1-X` snap cell (numbers.md "Operand order", T-3100).
- Engine: LOOKUP matches every cell. XMATCH/XLOOKUP compare at 15 digits: 332 known failures, owned by T-3132
  (functions-lookup, sonnet). Eval delta: +1 probe case, 786 cells, 453 pass, 333 new known failures (332 T-3132,
  1 T-3100), 0 regressions.
