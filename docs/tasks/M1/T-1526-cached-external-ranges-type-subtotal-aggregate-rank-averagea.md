---
id: T-1526
title: "Cached external ranges: TYPE, SUBTOTAL, AGGREGATE, AVERAGEA and legacy implicit intersection"
milestone: M1
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-1460]
---

## Goal
The last 7 cells of fixtures/external-link-cached-ranges.xlsx (excel-graph) pass: the functions
that take a reference, and legacy formulas, treat a cached range of a linked workbook
(`[1]Uses!$A$1:$B$3`) as Excel does.

## Context
Found in T-1460. That task makes a cached area a reference whose cells read the externalLink cache
(`external_links.rs`): aggregates get its cells as an array with reference-cell rules, and value
contexts get its values. These cells still fail (docs/excel-semantics/references.md "Cached values
of a linked workbook"):
- Refs!B62 `TYPE(…)` is 16 (engine 64: it sees an array);
- B59 `SUBTOTAL(9,…)` 15 and B60 `AGGREGATE(9,6,…!$A$1:$C$2)` 16 (engine `#VALUE!`: they need a
  reference and get an array; SUBTOTAL returns one `#VALUE!` on purpose, TODO(T-1526) in
  subtotal.rs, so no array reaches a scalar cell). RANK reads the cache since T-1460;
- B53 `AVERAGEA(…)` 3.2 (engine `#N/IMPL!`: `fn_averagea` has no array arm at all; MAXA works);
- B38 `SUM(IFERROR(…*1,100))`, B56 `SUMPRODUCT(IFERROR(…*1,100))` 100 and B44 `SUM(…!$A$2:$A$3+0)`
  `#VALUE!` (engine 123, 123, 6): in a legacy formula Excel implicitly intersects the external range
  as it does a range of this workbook. Check how T-1096's legacy parse handles `WrongRangeKind`.
- Narrow the array-bound DEVIATION (arrays.md, T-1460 review): aggregates that skip blanks
  (SUM, COUNT, MAX, AVERAGE, ...) could clip any cached area to the cache extent, so
  `SUM([1]Uses!$A$2:$B$1048576)` gives Excel's value instead of `#ERROR!`; only blank-sensitive
  readers (TEXTJOIN, CONCAT) need the full shape. Also probe COUNTA over a cached `t="str"` empty
  string and GCD/LCM over a cached range holding text.

## Acceptance criteria
- [ ] The 7 cells pass; `make eval-baseline` records them.
- [ ] Unit tests in `test_external_links.rs` cover each shape.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Refreshing links or opening the other workbook (directive 5).
