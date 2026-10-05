# xlsx storage quirks

## Control characters in text are stored as `_xHHHH_`
Characters invalid in XML 1.0 (U+0000–U+0008, U+000B, U+000C, U+000E–U+001F) are written as
`_xHHHH_` (hex, upper-case) in shared strings, inline strings and cached `<v>` of `t="str"`
cells; a literal `_xHHHH_` look-alike is written `_x005F_xHHHH_`.
Evidence: `ironcalc-calc/TEXT/CHAR.xlsx` (Excel-cached): `=CHAR(1)` is stored as `_x0001_`.
Readers must decode (`rustsheet_xlsx::import::decode_xlsx_escapes`); this is not CHAR behavior.

## Shared-formula members store no text; the master's text applies shifted
Only the master `<f t="shared" ref=.. si=..>` carries text; members are `<f t="shared" si=../>` and
compute the master's formula with every relative reference moved by (member − master) rows/columns
(absolute `$` parts stay). Evidence: poi/NumberFormatApproxTests.xlsx Tests!A96 is a member of si 5
whose master text is `TEXT(C41, B41)` and computes `TEXT(C96, B96)`. `rustsheet-eval show`/`snapshot`
print the shifted text (spacing kept) via `shift_formula_text`; `formula_kind` still carries `si`.
Spec: ECMA-376 Part 1 §18.3.1.40 (`f`). Excel never writes a member whose shifted references fall off
the sheet; `#REF!` for that case is only the harness's display choice, not observed Excel behavior.

## A hyperlink range above 10,000 cells links only its top-left cell
Rule: Excel applies a `<hyperlink ref="A1:XFD1048576">` to every cell of its range. The importer
expands a range of at most 10,000 cells to one link per cell; a larger range (a corrupt or
malicious file can name the whole sheet) gives only the top-left cell the link, to bound memory
(CLAUDE.md directive 5). Hyperlinks do not affect calculated values.
Evidence: none for Excel; this is a resource limit, not an Excel observation.
Status: DEVIATION (decision 0014)
Implemented in: crates/rustsheet-xlsx/src/import/worksheets.rs (`MAX_HYPERLINK_RANGE_CELLS`)
