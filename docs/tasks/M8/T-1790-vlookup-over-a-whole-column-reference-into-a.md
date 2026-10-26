---
id: T-1790
title: "VLOOKUP over a whole-column reference into a cached external workbook reads the cache, not #ERROR!"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
VLOOKUP whose table is a whole-column reference into a cached external workbook
(`'[1]Order Data'!$A:$B`) reads the externalLink cache and returns Excel's value instead of `#ERROR!`.

## Context
corpus-bench/56637 (115 failing cells, all from this): `Sheet1!B1` `=VLOOKUP(D1,'[1]Order Data'!$A:$B,2,FALSE)`
expected `"Period 3/ Wk3"`, engine `#ERROR!`; `Sheet1!B3:H3` `=VLOOKUP($B$1,'[1]Order Data'!$B:$J,n,FALSE)`
expected 300.5398, 261.7022, ... ; the 85 `#ERROR!` cells and 30 numeric mismatches downstream
(`B4 =B3/$I$3`, `B8 =COUNTIF(B12:B100,"1st Shift")` expected 21 got 0) follow from them. The workbook's
`xl/externalLinks/externalLink1.xml` caches sheet `Order Data` (sheetId 3) and uses the
`xxl21` extlinks2021 namespace. Excel for the web agrees with the cached values.
Find whether import drops this cache (e.g. the extlinks2021 markup or a `t="str"` cell), the whole-column
external area is not supported, or VLOOKUP does not take an external area; code: crates/rustsheet-engine/src/external_links.rs.
Inspect: `./target/release/rustsheet-eval show corpus-bench/56637/1_56637_input.xlsx`.
Related: T-1654 (3-D spans into a cached external workbook), T-0423 (cache indexing).

## Acceptance criteria
- [x] corpus-bench/56637's `F` line leaves eval/baseline/corpus-bench.txt with no new ones (or, if a residue remains, it is filed as a task with its cells).
- [x] A unit or eval test with a whole-column external VLOOKUP covers the fix.
- [x] `make check` green; reviewers approve.

## Out of scope
- Live links to other files (never: no I/O from evaluation).

## Outcome
Already fixed on main before this run picked it up; no code change. `rustsheet-eval show corpus-bench/56637/1_56637_input.xlsx`
on origin/main (5f9cac4 and later) is Pass, 144/144 checked, and eval/baseline/corpus-bench.txt holds only its `C` line
(no `F` line). The unit test `whole_column_lookups_keep_full_length` (crates/rustsheet-engine/src/test/test_external_links.rs)
covers VLOOKUP, MATCH and HLOOKUP over whole-column and whole-row references into a cached external sheet, exact and
approximate, against the same lookups over local columns. Eval delta: none.

