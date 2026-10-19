---
id: T-2020
title: "Fraction number formats: ?/?, ??/?? and ???/??? with and without a whole part (TEXT and displayed text)"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Number formats with a fraction whose denominator is written as up to three digit placeholders render
like Excel, in `TEXT()` and in the displayed text (`Workbook::formatted`): `?/?`, `??/??`, `# ?/?`,
`0 ?/?`, `# #/#`, `# ??/??`, `# ???/???`. Today the formatter has no fraction support at all
(`crates/rustsheet-engine/src/formatter/mod.rs`, "3.- You can use ?/? to format fractions": the lexer
reads `/` as a literal, so the value's digits are spread over the placeholders: `TEXT(PI(),"# ?/?")` is
"  /3", Excel "3 1/7").

## Context
Group (gap triage 2026-10-19): one root cause, the missing fraction renderer. TEXT is a catalog-M5
function (prior 2); the displayed text belongs to M10's exit (number-formats suite gen-m10).
- Value known failures, eval/baseline/gen-m10.txt (23 lines, all `TEXT(x,code)` on sheet T):
  039accabcfe1e6c7 T!A5, A11, A21; 03a284c57e24ff37 T!A10, A16, A26; 05c1c646c95a2d60 T!A3;
  0e3d8eba9432a18d T!A2, A23; 20512106c9574cdd T!A12, A16; 4398098060a7fa9d T!A26;
  4c6a49fe4a6fa404 T!A7; 659cd5776d423ce3 T!A23; 71ad5f93362f4ff4 T!A7; 810fee377a42575b T!A1, A18;
  9a8b7d68507d7ada T!A6; bfbcd16cee266336 T!A1, A11, A24; eec351e03ccedfe0 T!A8; fbdab487cb11409e T!A2.
  Examples (expected / ours): `TEXT(1.75,"?/?")` "7/4" / " /2"; `TEXT(12.345,"# ??/??")` "12 10/29" /
  "   /12"; `TEXT(-1234.5678,"# ??/??")` "-1234 46/81" / "- 12/35"; `TEXT(0.001,"# ???/???")`
  "0        " / "    /   ".
- Size M, so opus by the tier rubric even though the evidence is in eval/.
- Displayed text (report "Displayed text" section, not baselined, decision 0068): the 7 codes fail
  on 500 of their 540 gen-m10 text cells (`?/?` 75/80, `# #/#` 75/80, `??/??` 75/80,
  `# ?/?` 74/80, `0 ?/?` 74/80, `# ??/??` 73/80, `# ???/???` 54/60).
- Rules seen in the Excel values: docs/excel-semantics/number-formats.md "Fraction formats" (added with
  this task): improper fractions without a whole part (`?/?` on 123456.789 is "617284/5"), the
  denominator digit count as the bound, blank padding where the fraction is 0 (`# ?/?` on 1000 is
  "1000    "), `#` vs `0` vs `?` whole parts (`# #/#` on 0 is "0", `0 ?/?` on 0.125 "0 1/8", `# ?/?` on 1/3
  " 1/3"), the sign before the first part (`0 ?/?` on -0.5 "-0 1/2"), the right padding of a `?`
  denominator (`??/??` on 0.125 " 1/8 "). Which fraction is chosen is not "nearest with denominator below
  10^k": `# ???/???` on 0.001 is 0 although 1/999 is closer, and on 0.0049 it is 1/204. The last
  continued-fraction convergent whose denominator has at most k digits (no semiconvergents) picks the
  Excel fraction in all 488 numeric, non-overflow gen-m10 text cells of these codes; the nearest fraction with
  such a denominator picks 477 (triage check, 2026-10-19, a throwaway script over the case and oracle files).
- Code: `crates/rustsheet-engine/src/formatter/` (`lexer.rs` tokens, `parser.rs` sections, `format.rs`
  rendering). TEXT and the displayed text share it.
- Related: T-1711 owns probes/displayed-text-formats S!A17 (`# ?/?` on 12.5) and the round-half rule; it
  should pass here as a side effect. T-0982 (poi fraction formats with escaped literals, `=/=`, `_x`) and
  T-1740 (a numerator past the threshold shows 255 `#`) build on this renderer. Fixed denominators
  (`# ?/4`, `?/100`) are T-2021.

## Acceptance criteria
- [ ] The 23 gen-m10 lines listed above disappear from eval/baseline/gen-m10.txt (`make eval-baseline`)
      with no new F lines in any baseline.
- [ ] In the report's "Displayed text" section, the gen-m10 cells under the seven codes above pass,
      except overflow cells that show 255 `#` (T-1740) and any cell listed in the Outcome with its cause
      and a follow-up task.
- [ ] Unit tests in the formatter cover a whole part, no whole part, a zero fraction, a negative value
      and one value per denominator width.
- [ ] number-formats.md "Fraction formats" gains the layout rules the implementation needed, each with
      its cells, and its "Ours" line describes the implementation.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixed denominators (T-2021), the 255 `#` overflow (T-1740), literals between the parts (T-0982).
- Fraction formats in non-en-US locales.
