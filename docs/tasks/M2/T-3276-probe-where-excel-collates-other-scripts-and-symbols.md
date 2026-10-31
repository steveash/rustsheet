---
id: T-3276
title: "Probe where Excel collates other scripts and symbols: Hebrew, Arabic, Indic, Thai, Hangul, CJK ideographs, other-script digits and supplementary-plane characters"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
An excel-graph probe case shows whether Excel's text comparison orders the scripts beyond Latin, Greek,
Cyrillic and kana by code point after the letters (ours) or gives them structure of their own, and
coercion.md states what it found, so the engine's "every other character after the letters by code
point" rule is either VERIFIED or replaced by an engine task.

## Context
- coercion.md "Text comparison" (T-0220): "Every other character (CJK, other scripts, other symbols)
  keeps code-point order after the letters, UNVERIFIED". T-3114 probed Latin Extended-A, Greek and
  Cyrillic and left the rest to a follow-up (its Out of scope); T-3275 takes kana and the
  full-width/half-width block. No collation probe holds a Hebrew, Arabic, Indic, Thai, Hangul or CJK
  ideograph character today.
- Ours: `element` in crates/rustsheet-engine/src/functions/util/collation.rs weighs such a character
  `OTHER_BASE + code point`, after every letter; `punctuation` returns None above `€`, so symbols past
  it (`™`, `←`, `♥`, `✓`, `∞`) weigh by code point among the scripts; an astral character weighs its
  two UTF-16 units from `UNIT_BASE`, after every BMP character (only its place against lone
  surrogates and U+FFFD is probed: probes/text-lone-surrogates A18-A24). Combining marks outside
  U+0300-U+036F (Hebrew points, Arabic harakat, Devanagari matras and virama, Thai vowels and tone
  marks) are ordinary primaries in ours, never ignorable or accent-like.
- Questions the probe must answer, each with cells:
  1. Block order: one or two letters of each of Hebrew, Arabic, Devanagari, Thai, Hangul syllables
     and CJK ideographs, ranked against each other and against a ladder of `"z"`, `"ÿ"`, Greek `ω`,
     Cyrillic `я`: code-point order after the letters, or another block order.
  2. Order inside a script: does `"א" < "ב" < "ג"`, `"ا" < "ب"`, `"क" < "ख"`, `"ก" < "ข"`,
     `"가" < "나"`, `"一" < "丁"` follow code point (pick pairs where a radical/stroke or a
     dictionary order would disagree with the code point, for example CJK `"人"` vs `"一"`).
  3. Marks: a Hebrew point (`"ב"&UNICHAR(1468)`), an Arabic haraka (`"ب"&UNICHAR(1614)`), a Devanagari
     matra and virama, a Thai tone mark: equal to the bare letter, a later tiebreak, or a primary.
  4. Thai and Lao prevowels (`"เก"` vs `"กเ"`): reordered before comparison or not.
  5. Other-script digits (Arabic-Indic `٣`, Devanagari `३`, full-width digits are T-3275's): equal to
     or ordered with `"3"`, or after the letters.
  6. Symbols above `€` (`™`, `←`, `∞`, `♥`, `✓`, `☺`) and astral characters (`😀`, `𝐀`, CJK
     Extension B `𠀀`) against the scripts and the ASCII punctuation: where each block sits.
- Method: the `excel-probe` skill, excel-graph, laid out like probes/t3201-collation-wrapped-run-pairs
  (a Data column of strings and a Calc grid of `(x<y)*1+(x=y)*2+(x>y)*4` codes) plus
  `SUMPRODUCT(--(range<x))` ranks, `COUNTIF(range,"="&x)` and exact `MATCH`/`XMATCH` columns. Keep the
  case under about 1,500 formula cells. Milestone tag M2 (decision 0051).
- Code to read for the hypothesis, not to change here: `element`, `punctuation`, `UNIT_BASE` and
  `OTHER_BASE` in collation.rs.

## Acceptance criteria
- [x] A probe case `probes/t3276-collation-other-scripts` (milestone M2) is committed with its
      excel-graph oracle sidecar and answers each of questions 1-6 above with at least two cells.
- [x] coercion.md states the result for each question with the case id and cells (VERIFIED, or
      UNVERIFIED with the competing readings where the cells do not settle it), and the "Every other
      character ... UNVERIFIED" sentence is narrowed to what is still unprobed.
- [x] Every cell our engine fails is a known failure in eval/baseline/probes.txt (`make eval-baseline`,
      0 regressions), and an engine task filed in the same PR (`make new-task`, milestone M2, lane
      engine) lists them (case stem and A1 names in one bullet, decision 0159), so
      `make exit-owners MILESTONE=M2` reports 0.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing collation.rs (the engine task this files).
- Kana and full-width/half-width forms (T-3275).
- Right-to-left display, bidi controls (U+200E/U+200F) and zero-width characters: file a follow-up probe
  if the cells above suggest they matter.

## Outcome
probes/t3276-collation-other-scripts (excel-graph, 2026-10-31; 99 strings ranked, with `=` counts, exact
MATCH, XMATCH, COUNTIF, 36 chosen pairs and 13 single cells; 643 checked cells). coercion.md gains
"Other scripts, their marks and digits, symbols and astral characters (T-3276)":
- Block order is not code-point order: kana sort before Hebrew, and the astral `𝐀`/`😀` sit between
  Lao and Hangul. Inside each script the order is code-point order.
- Hebrew points, Arabic harakat and the Thai tone mark are accents; Devanagari matras and the virama
  are letters.
- Prevowels are not reordered: Thai `เ`/`แ` sort before the consonants, Lao `ເ` after them.
- Other-script digits are a tiebreak on their ASCII digit.
- Symbols above `€` sort among the punctuation.
- In passing: VALUE reads Arabic-Indic, Devanagari and Thai digits as numbers.
Eval delta: +1 case; 115 new known failures, 0 regressions. Owners: T-3324 (101 collation cells) and
T-3325 (14 digit cells).
