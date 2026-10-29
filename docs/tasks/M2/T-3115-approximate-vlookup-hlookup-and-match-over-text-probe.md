---
id: T-3115
title: "Approximate VLOOKUP, HLOOKUP and MATCH over text: probe whether the search orders by the collation or by the exact-match key, and match it"
milestone: M2
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
VLOOKUP/HLOOKUP with range_lookup TRUE and MATCH with match_type 1 and -1 compare text keys the way Excel does.
The cases that matter are texts that the collation calls equal or reorders, but that exact match tells apart: `"æ"`
against `"ae"`, `"ß"` against `"ss"`, `"a-e"` against `"ae"`, and accented letters. The answer comes from a committed
oracle case, not from inference.

## Context
- Exact match does **not** use the collation: `MATCH("ae",{"æ"},0)` and `VLOOKUP("ss",{"ß"},1,FALSE)` are `#N/A`
  (probes/text-collation-lookups, excel-graph 2026-10-15, T-1590; docs/excel-semantics/coercion.md "Collation equality in
  lookups and criteria"). The engine implements that with `ExactKey` (case-folded UTF-16 units) in
  crates/rustsheet-engine/src/functions/lookup_and_reference/lookup_table.rs.
- The approximate searches (`SearchVector::approximate_search`, `descending_search`) compare text through
  `compare_searched` → `compare_values` → `cmp_text_excel`, the collation. lookup.md "Approximate match" only says
  "Text comparison uses Excel's collation, not code-point order (T-0220)". That came from ordinary ASCII text. No
  probe puts collation-equal or hyphen-skipped texts in an approximate search, so Excel could use the exact-match key
  in its binary search instead.
- Why opus: the search is Excel's type-skipping binary search (T-0212). The probe tables must be built so that the
  midpoints visited separate three models: (a) the collation with collation-equal runs (`"ae"` lands on the last of
  `"ae","æ"`), (b) the collation with ties broken by code point, (c) a case-folded ordinal order like `ExactKey`. Equal
  runs, a hyphen inside a word (`"co-op"` against `"coop"`, `"cop"`) and an accent tiebreak (`"résumé"` against
  `"resume"`, `"resumf"`) each need tables of length 1 to 5, so that the binary search order is known.
- Method: the `excel-probe` skill (excel-graph). One case with VLOOKUP, HLOOKUP, MATCH(…,1) and MATCH(…,-1) over the
  same tables, so the four can be compared.

## Acceptance criteria
- [ ] A new probe case `probes/t3115-approximate-text-collation` (oracle-filled) covers VLOOKUP TRUE, HLOOKUP TRUE,
      MATCH 1 and MATCH -1 with keys and tables built from `æ`/`ae`, `ß`/`ss`, `œ`/`oe`, `co-op`/`coop`, `'a`/`a`,
      `résumé`/`resume` and case pairs (`"A"`/`"a"`). It has at least one table per model above in which the models
      give different results.
- [ ] lookup.md "Approximate match" says which model Excel uses, with the case id and cells (VERIFIED). The
      inference sentence is replaced.
- [ ] Every cell of the new case passes. If Excel does not use the collation, the approximate searches compare text
      through one shared comparator (reuse `ExactKey`'s folding, do not copy it). probes/t3073-lookup-near-equal-approximate,
      probes/text-collation-lookups and the gen-m2 VLOOKUP/HLOOKUP/MATCH cells show 0 regressions (`make eval-check`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- LOOKUP, XLOOKUP and XMATCH approximate and binary modes. File a follow-up (M6/M7) if the answer suggests they
  differ. T-3106 covers their number keys.
- New collation weights for letters beyond Latin-1 (T-3114).
- Ordered criteria operators (T-3103).
