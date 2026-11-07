---
id: T-4807
title: "Pivot date items show in the source field's date number format"
milestone: M16
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A pivot over a field of dates shows its date items in the source cells' date number format, as Excel does.

## Context
Found by T-2085: on probes/t2085-getpivotdata-items sheet D, rows When over date serials formatted m/d/yyyy,
Excel shows the items as 1/1/2024, 1/2/2024 and 2/1/2024, while the engine shows 45292, 45293 and 45323. The values
match; only the shown text differs. See pivots.md "Item casing when cache and source disagree, and date items" for
what is known about date items.

Owner of these visibly-off cells (decision 0245):
- probes/t2085-getpivotdata-items D!A4:A6

## Acceptance criteria
- [x] The cells above pass, and no other case regresses (`make eval-baseline`).
- [x] pivots.md "Item casing when cache and source disagree, and date items" says the engine now matches.
- [x] `make check` green; reviewers approve.

## Out of scope
- GETPIVOTDATA with date items (T-4805).

## Outcome
The Goal's premise was not quite Excel's: a date item does not take the source cells' format but a fixed
short date, `m/d/yyyy`, with ` h:mm` when the item has a time part. probes/t4164-pivot-date-times (source
`m/d/yyyy h:mm`) shows midnight as `4/17/2024`, not `4/17/2024 0:00`; t2085 D and t4130-pivot-date-filters
(source `m/d/yyyy`) show `1/1/2024`. So no source format is carried in the cache (the handoff's plan is not
needed): `RenderedCell::number_format` is now a `CellFormat` (a data field's `numFmtId`, or a code), and a
`CacheItem::Date` label cell gets `date_item_format` (pivot/layout/render.rs); the grid writer applies a code
as it applied an id. The probe-case test model now applies each sheet's `number_formats` and typed error
cells, so `date_item_labels_show_as_short_dates` (pivot/grid/tests.rs) refreshes date-formatted sources as
Excel does. pivots.md "Item casing when cache and source disagree, and date items" records the rule and its
UNVERIFIED edges (seconds, time-only items, date subtotal labels, a selected date page item).

Eval delta: probes text failures 754 -> 640 (114 fixed: t2085-getpivotdata-items D!A4:A6, 86 in
t4130-pivot-date-filters, 25 in t4164-pivot-date-times), 0 regressions; value known failures unchanged.

## Handoff (resumed)
Tried (sonnet run): read the render path. Date labels render in `item_cell` (crates/rustsheet-engine/src/pivot/layout/render.rs:125) as
`PivotCell::Number`; `RenderedCell.number_format` is a `numFmtId` applied by `set_pivot_number_format` (pivot/grid.rs ~700), so
label cells could already carry a format. The blocker: `CacheField` (pivot.rs:104) keeps no source number format, and
`refresh.rs::read_source` / `is_date_style` (~line 105-130) only tests date-ness through `SourceCells::value`.
Plan: add `number_format: Option<u32>` to `CacheField` (OOXML `cacheField numFmtId`), fill it in `read_source` from the first date
source cell's xf `num_fmt_id` (extend `SourceCells`), render date-item labels with it via `put_formatted`, and read it on
xlsx import. About 25 `CacheField { .. }` construction sites need the new field (`..Default` where possible). Persisted model plus
trait change, so judged beyond sonnet. Nothing implemented; `make eval-show` was not completed.
