---
id: T-3321
title: "Collate full-width forms, kana, voicing marks and the long-vowel and iteration marks as Excel does (T-3275 probes)"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`cmp_text_excel` orders and equates full-width/half-width forms, kana, the voicing marks and the
long-vowel and iteration marks as Excel does, so `=`/`<`, the criteria, MATCH/VLOOKUP and the
XMATCH/XLOOKUP `CollationKey` pass both T-3275 probe cases.

## Context
- Rules and cells: docs/excel-semantics/coercion.md "Full-width and half-width forms, kana, voicing
  and the long-vowel mark (T-3275)". In short: width, kana type, small kana and voicing are tiebreaks
  on a shared letter (voicing at the accent level; small kana and `ー` before kana type; width and kana
  type last before the ignorables); kana sort in syllabary order after Cyrillic and before the CJK
  ideographs; `ー`/`ゝ`/`ゞ`/`々` repeat the preceding kana or kanji, and alone sort after the ideographs,
  equal to each other; U+3099, U+309B and U+FF9E voice the preceding character; U+FF07 and U+FF0D are
  ignored like `'` and `-`.
- Ours today: `element` in crates/rustsheet-engine/src/functions/util/collation.rs maps all of these
  to `single(OTHER_BASE + code point)`. The width rule has the shape of `superscript` there.
- `ー` after a Latin letter is not transitive in Excel (`"aー"` vs `"Ａa"`, `"ab"`); a sort key cannot
  match every pairs cell. Decide here whether to match the pairwise `<` (a comparator special case)
  or to leave those cells as known failures with the reason in coercion.md; either way the
  XMATCH/XLOOKUP `CollationKey` must stay a total order.
- Exact MATCH quirks: MATCH of `"aー"` finds `"aa"`, and of `ゝ`/`ゞ`/`々` finds `ー` (case 1 C127:C129,
  C136); MATCH does not equate the decomposed `か`+U+3099 with `が` (as for Latin marks).
- Failing cells (172 + 358):
  - probes/t3275-collation-width-kana.case.json Calc!A2:A5, Calc!A7:A120, Calc!A122:A137, Calc!B96:B97, Calc!B106, Calc!B111, Calc!B119, Calc!B127:B129, Calc!C127:C129, Calc!C136, Calc!D106, Calc!D111, Calc!D127:D129, Calc!E96:E97, Calc!E106, Calc!E111, Calc!E119, Calc!E127:E129, Calc!G3:G5, Calc!G7, Calc!G12:G17, Calc!G21, Calc!G24, Calc!G34
  - probes/t3275-collation-width-kana-pairs.case.json Calc!B1:AL37 (the whole grid; 358 of its 1,369 cells fail, listed in
    eval/baseline/probes.txt)

## Acceptance criteria
- [ ] Every cell of probes/t3275-collation-width-kana passes, and every cell of
      probes/t3275-collation-width-kana-pairs outside the non-transitive `"aー"` row and column
      (Calc!I1:I37, Calc!B8:AL8) passes; those are either matched or documented as known failures.
- [ ] No previously passing collation probe regresses (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other scripts and CJK ideograph order (T-3276).
- UPPER/LOWER/ASC/DBCS conversions.
