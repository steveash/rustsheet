---
id: T-3275
title: "Probe how Excel collates full-width and half-width forms and kana (width, hiragana/katakana, small kana, voicing marks, the long-vowel mark)"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
An excel-graph probe case pins where Excel's text comparison puts full-width and half-width forms and
Japanese kana relative to the letters, digits and punctuation they mirror, and coercion.md states the
rule (or rules) precisely enough for an engine task to implement, replacing today's UNVERIFIED
"every other character keeps code-point order after the letters".

## Context
- coercion.md "Text comparison" (the fitted key, T-0220) ends with: "Every other character (CJK, other
  scripts, other symbols) keeps code-point order after the letters, UNVERIFIED". T-3114's Out of scope
  left CJK, kana and the other scripts to a follow-up probe "if the evidence suggests structure beyond
  code-point order"; no probe since has used a full-width form or a kana (checked: no
  probes/*collation* case holds a character of U+3040-U+30FF or U+FF00-U+FFEF).
- Ours: `element` in crates/rustsheet-engine/src/functions/util/collation.rs maps such a character to
  `single(OTHER_BASE + code point)`, so `"Ａ"` (U+FF21) sorts after every Latin letter and is never
  equal to `"A"`; `"ア"` and `"あ"` are distinct primaries; the half-width `"ｱ"` (U+FF71) sorts after
  both. The same comparison serves `=`/`<`, the criteria (COUNTIF `"=Ａ"`), MATCH/VLOOKUP approximate
  search and the XMATCH/XLOOKUP `CollationKey`, so the rule matters to every M2 consumer.
- The engine already treats some compatibility forms as a variant of their base (superscript digits,
  `ª`, `º`: `superscript` in collation.rs), which is the shape a width rule would take if Excel ignores
  width at the primary level.
- Questions the probe must answer, each with cells:
  1. Width: full-width Latin letters (`Ａ`, `ａ`, `Ｚ`), digits (`０`-`９`), punctuation and space
     (U+FF01-U+FF0F, U+3000) against their ASCII forms: equal (`=`), a later tiebreak (between `"A"`
     and `"B"`, ranked against `"Á"`), or after the letters (ours).
  2. Kana type: hiragana against katakana (`あ`/`ア`, `か`/`カ`), half-width katakana (`ｱ`, `ｶ`) against
     both; whether they are equal, a tiebreak, or distinct letters, and the base order of the gojuon
     rows (`あ < い < う ...`; does `ア < イ` follow code point or the syllabary).
  3. Small kana and voicing: `ぁ`/`あ`, `っ`/`つ`, `が`/`か`, `ぱ`/`ば`/`は`, and the decomposed voicing
     `か`&UNICHAR(12441) against `が` (canonical equivalence, as T-3114 found for Latin marks).
  4. The long-vowel mark `ー` (U+30FC) and the iteration marks `ゝ`/`ゞ`/`々`: a letter of their own,
     a repeat of the preceding vowel, or punctuation.
  5. Where the whole block sits: kana and full-width forms ranked against a ladder of `"z"`, `"ÿ"`,
     a Greek letter, a Cyrillic letter and a CJK ideograph (`一`, `中`).
- Method: the `excel-probe` skill, excel-graph, laid out like probes/t3201-collation-wrapped-run-pairs
  (a Data column of strings and a Calc grid of `(x<y)*1+(x=y)*2+(x>y)*4` codes) plus
  `SUMPRODUCT(--(range<x))` ranks, `COUNTIF(range,"="&x)` and exact `MATCH`/`XMATCH` columns, since
  T-3114 found MATCH and XMATCH disagree on canonical equivalence. Keep the case under about 1,500
  formula cells (two cases if needed). Milestone tag M2 (decision 0051; collation is the comparison
  every M2 consumer shares).
- Code to read for the hypothesis, not to change here: `element`, `fold`, `decompose` and `superscript`
  in collation.rs, `DECOMPOSED` in collation_tables.rs.

## Acceptance criteria
- [ ] A probe case `probes/t3275-collation-width-kana` (milestone M2) is committed with its
      excel-graph oracle sidecar and answers each of questions 1-5 above with at least two cells.
- [ ] coercion.md gains a section stating the width and kana rules with the case id and cells, each
      marked VERIFIED or, where the cells do not settle it, UNVERIFIED with the competing readings; the
      "Every other character ... UNVERIFIED" sentence is narrowed to what is still unprobed.
- [ ] Every cell our engine fails is a known failure in eval/baseline/probes.txt (`make eval-baseline`,
      0 regressions), and an engine task filed in the same PR (`make new-task`, milestone M2, lane
      engine) lists them (case stem and A1 names in one bullet, decision 0159), so
      `make exit-owners MILESTONE=M2` reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing collation.rs (the engine task this files).
- Scripts other than kana and the full-width/half-width block (T-3276: Hebrew, Arabic, Indic, Thai,
  Hangul, CJK ideograph order, symbols and supplementary-plane characters).
- Case mapping in UPPER/LOWER/PROPER and ASC/DBCS conversions (text.md).
