---
id: T-0280
title: "Probe typed cell input of (1) and (1,000) needs a typed-entry oracle path"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for the value and number format Excel stores when `(1)`, `(1,000)`, `( 1 )`, `(-1)`, `($1)`, `(1%)` are typed into a cell.

## Context
T-0277 could only probe text coercion: case-file strings are written as text cells, so Excel never parses them as typed input. Needs an oracle path that enters values as typed (e.g. Graph range `values` writes, or a `typed` cell tag in spec2xlsx/oracle fill) and reads back `numberFormat`; then compare with the engine's `set_user_input` (-1 with no format, -1000 with `#,##0`).

## Acceptance criteria
- [x] Oracle can enter typed input and return its number format; a probe case holds the six forms; coercion.md records them; an engine task is filed if the engine differs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Spec cells `{"entered":"text"}` are typed entry: spec2xlsx leaves the cell blank and records it in
docProps custom properties `rustsheet.entered.<n>`; the excel-graph backend writes each with a
range `values` PATCH before calculating and logs the `numberFormat` Excel picked (`ENTERED` on
stderr); excel-desktop sets `Range.Value`; libreoffice refuses; the engine adapter uses
`Input::Parse`. probes/typed-paren-number-input (excel-graph) holds the six forms plus controls:
values all match the engine (24/24 cells pass). Formats (coercion.md): `(1)` General, `(1,000)`
`#,##0`, `($1)` `$#,##0_);[Red]($#,##0)`, `(1%)` `0%`; the engine's currency and percent formats
differ, filed as T-0283. `CELL("format")` is `#VALUE!` on excel-graph, so formats are logged, not
scored. Eval delta: +1 probe case, +24 passing cells, 0 regressions.
