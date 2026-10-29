---
id: T-3084
title: "Probe whether criteria ordering operators (<, <=, >, >=) compare numbers at 15 digits or exactly, as = and <> do"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for whether COUNTIF, SUMIF, COUNTIFS and SUMIFS compare a numeric cell with an ordering criterion
(`"<1"`, `"<=1"`, `">1"`, `">=1"`) on the 15-digit values, as their `=` and `<>` criteria do, or exactly, so that a
cell holding `1+2^-52` is not at once "equal to 1" and "greater than 1" in the engine unless Excel says so.

## Context
- docs/excel-semantics/numbers.md "Criteria equality": numeric criteria equality compares the 15-digit values
  (probes/tiny-vs-zero-criteria: over B1:B5 = 1E-300, 0, 1E-17, 2E-16, `=1+2^-52`, `COUNTIF(B1:B5,1)` is 1 and
  `"<>1"` is 4, A9, A10). No probe has an ordering criterion against a value a few ULP from its operand: the only
  ordering cells there are `">0"` and `"<=0"` against tiny values (A11, A12), where both readings agree, and
  t3048-snap-conditional-sums uses `"<2"` far from every cell.
- The engine is split: crates/rustsheet-engine/src/functions/criteria.rs `numbers_equal` (`cmp_excel_precision`)
  serves `Eq`/`Ne`, but `result_is_less_than_number`, `result_is_less_or_equal_than_number`,
  `result_is_greater_than_number` and `result_is_greater_or_equal_than_number` compare the raw doubles. So today
  `COUNTIF(B5,"=1")` is 1, `COUNTIF(B5,">1")` is 1 and `COUNTIF(B5,"<=1")` is 0. The comparison operators of a
  formula use 15 digits throughout (numbers.md "Comparison tolerance", `compare_values` in functions/util.rs), but
  that entry notes `<` and `>=` were uninformative (X >= b in every probe cell), so even `=1<1+2^-52` lacks a cell.
- An ordering criterion's operand is text, cut to 15 digits when parsed (numbers.md "15-digit literal parsing";
  `"<="&C1` prints C1 at 15 digits), so the open question is the cell side only.
- Wrong either way is visibly off (a count or sum changes by a whole cell). Opus: the 15-digit cut is not a
  relative epsilon (numbers.md "Comparison tolerance": k = 3-8 ULP is equal at some bases only), so the probe needs
  ladders and a pair straddling a 15th-digit rounding boundary to tell "15-digit values", "exact" and "a tolerance"
  apart, and the follow-up touches the shared criteria grammar every *IF(S) and D-function consumer uses.

## Acceptance criteria
- [ ] A new excel-graph probe case `probes/t3072-criteria-near-equal-order` (milestone M2, decision 0043; not M1)
      reads back COUNTIF, SUMIF (with a sum_range), COUNTIFS and SUMIFS with each of `"<1"`, `"<=1"`, `">1"`,
      `">=1"`, `"=1"`, `"<>1"` over cells built by formula (literals are cut to 15 digits): `=1+k*2^-52` for
      k = 1, 2, 3, 8, `=1-k*2^-53` for k = 1, 2, 3, 8, and a pair a few ULP apart on either side of a 15th-digit
      rounding boundary, each in a one-cell range so every count is 0 or 1. Control rows give the plain operators on the
      same cells with the cell on each side (`=C1<1`, `=1<C1`, `=C1>=1`, `=1>=C1`).
- [ ] numbers.md "Criteria equality" (or a new section beside it) and docs/excel-semantics/criteria.md record, per
      operator, whether the cell side is compared at 15 digits or exactly, citing the cells; the "Comparison
      tolerance" entry gains the `<`/`>=` control cells.
- [ ] Every cell the engine does not match is a known failure in eval/baseline/probes.txt, and one follow-up task
      (milestone M2, lane functions-criteria, tier opus) names those cells and the criteria.rs functions to change;
      `make exit-owners MILESTONE=M2` reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the follow-up task owns them).
- Approximate lookups (VLOOKUP/HLOOKUP/MATCH with a non-zero match type): their own probe task.
- Conditional-format value operators (T-1051, M14) and D-function criteria ranges (M9).
- Text and boolean ordering criteria.
