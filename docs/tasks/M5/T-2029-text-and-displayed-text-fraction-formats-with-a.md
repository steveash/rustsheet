---
id: T-2029
title: "TEXT and displayed text: fraction formats with a fixed denominator (# ?/4, # ??/16, ?/100): integer part, sign, padding"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The formatter renders fraction number formats whose denominator is a literal number (`# ?/2`, `# ?/4`,
`# ?/10`, `# ??/16`, `?/100`) as Excel does, in TEXT() and in a cell's displayed text: the optional
integer part, the rounded numerator over the fixed denominator, the sign, and the blank padding when
the fraction part rounds away. This task also lays down the fraction section itself (lexer token,
parsed part, renderer) that T-2030 extends to placeholder denominators.

## Context
The engine has no fraction support at all: `/` is a literal and the `?`/`#`/`0` around it are plain
digit placeholders, so every fraction code prints garbage (`TEXT(36.5,"# ?/2")` is "3 6/2", Excel
"36 1/2"). Gap triage 2026-10-19: in suite gen-m10 (excel-graph `Range.text`, 2026-10-16, T-1611)
**all 829 numeric displayed-text cells under the 12 fraction codes fail**; 329 of them use the five
fixed-denominator codes below (314 values, 15 overflow cells that are T-1740's).

Rules read from the cells (recorded in docs/excel-semantics/number-formats.md, "Fraction formats",
added by this triage), `F!` cells of the named gen-m10 case:
- The numerator is the fraction part times the denominator, rounded half away from zero, never
  reduced: `# ?/4` on 1.5 "1 2/4" (039accab F7), `# ??/16` on 1.5 "1  8/16" (05c1c646 A3), `# ?/10`
  on 36.5 "36 5/10" (43980980 P2), 0.125 under `# ?/4` " 1/4" (039accab F12: 0.5 rounds up).
- A numerator that rounds to the denominator carries into the integer part: `# ?/2` on 1.75 "2    "
  (bfb1a4b0 S16), 45000.75 "45001    " (8e073c5b I13); `# ?/4` on 0.9999999999 "1    " (039accab F3).
- A numerator of 0 blanks the whole fraction part to spaces of its width, and the integer part then
  prints even under `#`: `# ?/4` on 1E-5 "0    " (F4), `# ??/16` on 61 "61      " (05c1c646 A5),
  `# ?/2` on 0.125 "0    " (43980980 N10), on 0 "0    " (N5).
- A zero integer part under `#` is left out, with its space kept: `# ?/4` on 0.5 " 2/4" (039accab F8),
  `# ?/10` on 0.125 " 1/10" (43980980 P10).
- The minus leads; it stays on a blank-integer and on a rounded-to-zero value: `# ?/4` on -0.5 "- 2/4"
  (9a8b7d68 A1), `# ??/16` on -1/7 "-  2/16" (05c1c646 A14), `# ?/2` on -1/7 "-0    " (bfb1a4b0 S4).
- `?` pads the numerator on the left: `# ??/16` on 1.5 "1  8/16", on 2/3 " 11/16" (05c1c646 A19).
- With no integer part the whole value goes into the numerator: `?/100` on 1.5 "150/100", 61
  "6100/100", -1/7 "-14/100", 0 "0/100" (05c1c646 D3, D5, D14, bfbcd16c Y17).
TEXT() on the same codes (sheet T) equals the displayed text; 16 of its cells are value known failures
in eval/baseline/gen-m10.txt: 039accabcfe1e6c7 T!A6; 03a284c57e24ff37 T!A11; 05c1c646c95a2d60 T!A1,
T!A4; 4398098060a7fa9d T!A14, T!A16; 8e073c5b2145567a T!A11, T!A22; 9a8b7d68507d7ada T!A1;
bfb1a4b09b0c998e T!A19; bfbcd16cee266336 T!A22, T!A25; e1c29d3fe448682c T!A24, T!A26;
eec351e03ccedfe0 T!A6, T!A9. Example: `TEXT(1E+15,"# ?/4")` is "1000000000000000    ", ours
"100000000000000 0/4".

Code: crates/rustsheet-engine/src/formatter/ (`lexer.rs` tokens, `parser.rs` `ParsePart::Number`,
`format.rs` rendering; shared by TEXT and `Workbook::formatted`). Reuse the existing half-away
rounding helper (`decimal_round::round_decimal` with `RoundMode::HalfAwayFromZero`, as general.rs does),
do not add another. Show cases with
`make eval-show CASE=gen-m10/<id>.case.json`; the report's "Displayed text" section lists every text cell.

## Acceptance criteria
- [ ] The 16 TEXT() lines listed above leave eval/baseline/gen-m10.txt, with no new F lines in any baseline.
- [ ] The 314 `TF` lines of eval/baseline/gen-m10.txt for `F!` cells under `# ?/2`, `# ?/4`, `# ?/10`,
      `# ??/16` and `?/100` leave it (`make eval-baseline`); only the 15 overflow cells whose Excel text
      is 255 `#` stay (T-1740).
- [ ] Formatter unit tests for one cell of each rule above.
- [ ] docs/excel-semantics/number-formats.md "Fraction formats" says what is implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Placeholder denominators (`?/?`, `??/??`, `# #/#`, `0 ?/?`): T-2030.
- Numerator overflow to 255 `#` (`?/100` on 1000): T-1740.
- Escaped literals, `=/=` separators and `_x` spacers between the parts (POI NumberFormatTests): T-0982.
- Fractions inside date/time sections, and `General`/scientific next to a fraction.
