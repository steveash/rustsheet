# xlsx storage quirks

## Control characters in text are stored as `_xHHHH_`
Characters invalid in XML 1.0 (U+0000–U+0008, U+000B, U+000C, U+000E–U+001F) are written as
`_xHHHH_` (hex, upper-case) in shared strings, inline strings and cached `<v>` of `t="str"`
cells; a literal `_xHHHH_` look-alike is written `_x005F_xHHHH_`.
Evidence: `ironcalc-calc/TEXT/CHAR.xlsx` (Excel-cached): `=CHAR(1)` is stored as `_x0001_`.
Readers must decode (`rustsheet_xlsx::import::decode_xlsx_escapes`); this is not CHAR behavior.
