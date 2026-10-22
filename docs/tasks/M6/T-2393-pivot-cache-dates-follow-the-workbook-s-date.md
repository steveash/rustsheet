---
id: T-2393
title: "Pivot cache dates follow the workbook's date system (1904 workbooks)"
milestone: M6
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-2082]
---

## Goal
In a `date1904` workbook, the `<d>` items of an imported pivot cache hold the same serials as the cells they came from.

## Context
`crates/rustsheet-xlsx/src/import/pivots.rs` `cache_item` reads `<d v>` with `iso8601_to_serial`, which gives a 1900-system serial; in a 1904 workbook that is 1462 days off from the cell values (T-2082 review). No answer depends on it yet (GETPIVOTDATA matches text items only), but it must be right before dates are matched. See how `sheet_data.rs` reads `t="d"` cells under `date1904` and reuse that conversion (one definition); pass the date system into `load_pivots`. docs/excel-semantics/web.md "GETPIVOTDATA against a pivot table" notes the gap.

## Acceptance criteria
- [ ] A unit test in `import/pivots/tests.rs`: a 1904 workbook's `<d v="2024-01-01T00:00:00"/>` item is `CacheItem::Date(43830.0)`, a 1900 workbook's is 45292.0.
- [ ] The web.md note on 1904 dates is removed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Matching dates in GETPIVOTDATA (T-2085 decides the rule).
