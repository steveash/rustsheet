---
id: T-2030
title: "TEXT and displayed text: fraction formats with placeholder denominators (?/?, ??/??, ???/???) pick Excel's fraction"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2029]
---

## Goal
Fraction formats whose denominator is placeholders (`?/?`, `??/??`, `# ?/?`, `# ??/??`, `# ???/???`,
`0 ?/?`, `# #/#`) choose the same fraction Excel does and lay it out like Excel, in TEXT() and in a
cell's displayed text, on top of the fraction section T-2029 adds.

## Context
Gap triage 2026-10-19: in suite gen-m10 (excel-graph `Range.text`, 2026-10-16, T-1611) all 500
displayed-text cells under these seven codes fail (488 values, 12 overflow cells that are T-1740's);
the engine has no fraction support (see T-2029). Rules in docs/excel-semantics/number-formats.md
"Fraction formats" (added by this triage); `F!` cells of the named gen-m10 case:
- **Which fraction.** Lead, not yet implemented: Excel's fraction is the *last continued-fraction
  convergent* of the fraction part whose denominator fits the placeholders (at most 9, 99, 999). It
  matches all 317 gen-m10 text cells that show an `n/d` (scratch check during triage), while the
  closest fraction within the bound matches 308: `# ??/??` on PI() is "3  1/7 " (4c6a49fe G13: 14/99 is closer,
  the next convergent 15/106 is too wide; `# ???/???` gives "3  16/113", 05c1c646 C18), and on
  123456.789 "123456 15/19" (20512106 L16, 4c6a49fe G8, 659cd577 W3: 71/90 is closer). 12.345 is
  "12 10/29" under `# ??/??` (20512106 L7), "12  69/200" under `# ???/???` (05c1c646 C11) and "37/3"
  under `?/?` (43980980 Z16). The stopping rule on binary
  noise (123456.789's fraction part is 0.78900000000430737...) is the part to fit: state the rule
  you implement and the cells that decide it.
- **Improper fractions without an integer part:** `?/?` on 1.5 "3/2", 36.5 "73/2", PI() "22/7",
  1234567890 "1234567890/1", 0 and 1E-5 "0/1", 0.9999999999 "1/1", -1 "-1/1" (039accab E-column,
  43980980 Z-column); `??/??` on 123456.789 "2345679/19", on -1.5 "- 3/2 " (03a284c5 Z4, 039accab U1).
- **Padding:** `?` pads the numerator on the left and the denominator on the right:
  `# ???/???` on 1.5 "1   1/2  ", on 0.0049 "   1/204" (05c1c646 C3, bfbcd16c X4); `??/??` on 0.5
  " 1/2 ". `#` does not pad: `# #/#` on 0.5 "1/2", on 1234567890 "1234567890" (039accab K8, K2).
- **Zero fraction part:** blanked to the fraction's width and the integer printed even under `#`:
  `# ?/?` on 0.0049 "0    ", on 999.9999 "1000    " (0e3d8eba F18, F5), `# ???/???` on 1 "1        ".
- **Zero integer part:** `#` leaves it out (`# ?/?` on 0.125 " 1/8", 20512106 P1), `0` prints it
  (`0 ?/?` on 1/3 "0 1/3", on -1/7 "-0 1/7"; 0e3d8eba W4, 71ad5f93 G7); the minus leads either way
  (`# ?/?` on -0.5 "- 1/2", 0e3d8eba F9).
TEXT() on the same codes (sheet T) equals the displayed text; these 23 value known failures are in
eval/baseline/gen-m10.txt: 039accabcfe1e6c7 T!A5, A11, A21; 03a284c57e24ff37 T!A10, A16, A26;
05c1c646c95a2d60 T!A3; 0e3d8eba9432a18d T!A2, A23; 20512106c9574cdd T!A12, A16; 4398098060a7fa9d T!A26;
4c6a49fe4a6fa404 T!A7; 659cd5776d423ce3 T!A23; 71ad5f93362f4ff4 T!A7; 810fee377a42575b T!A1, A18;
9a8b7d68507d7ada T!A6; bfbcd16cee266336 T!A1, A11, A24; eec351e03ccedfe0 T!A8; fbdab487cb11409e T!A2.
Example: `TEXT(PI(),"# ?/?")` is "3 1/7", ours "  /3".
Also probes/displayed-text-formats S!A17 (`# ?/?` on 12.5 is "12 1/2", a `TF` line in
eval/baseline/probes.txt; moved here from T-1711 by this triage).

Code: crates/rustsheet-engine/src/formatter/ (the fraction part T-2029 adds). The convergent search
is bounded (at most ~45 steps for a double); no allocation per step.

## Acceptance criteria
- [ ] The 23 TEXT() lines above leave eval/baseline/gen-m10.txt, and the `TF` line
      probes/displayed-text-formats S!A17 leaves eval/baseline/probes.txt; no new F or TF lines.
- [ ] The 488 `TF` lines of eval/baseline/gen-m10.txt for `F!` cells under the seven codes above
      leave it; only the 12 overflow cells (255 `#`, T-1740) stay. If a few cells resist, list them with
      Excel's and our text in the Outcome and file a follow-up.
- [ ] The fraction-choice rule, its stopping rule and the cells that decide them are in
      docs/excel-semantics/number-formats.md "Fraction formats"; unit tests for PI() under `# ??/??`,
      123456.789 under `# ??/??` and 12.345 under `# ???/???`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixed denominators (T-2029); numerator overflow to 255 `#` (T-1740); escaped literals and `=/=`
  between the parts (T-0982).
