---
id: T-3399
title: "Probe the collation points T-3276 left open: Bengali, Tamil, Georgian, Armenian and Ethiopic letters and digits, the trademark and infinity signs, other astral planes and emoji sequences, other Thai and Lao vowels"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3324, T-3325]
---

## Goal
An excel-graph probe case settles every collation and digit question docs/excel-semantics/coercion.md
still marks UNVERIFIED after T-3276, so the M2 text-comparison rule has evidence for each script and symbol
block the engine places, and each mismatch is owned by an engine task.

## Context
- coercion.md "Text comparison" ends: "Every other character (other scripts, such as Bengali, Tamil,
  Georgian, Armenian and Ethiopic, and other symbol blocks) keeps code-point order after the letters,
  UNVERIFIED." The T-3276 section ("Other scripts, their marks and digits, symbols and astral characters")
  closes with: "UNVERIFIED: whether `™` is an expansion (`"tm"`) or a symbol placed between `a` and `y`
  (only `"Ab"` and `"ÿ"` bracket it); where `∞` sits against the punctuation other than digits; where other
  astral characters (other planes, emoji with modifiers) sit; whether Thai vowels other than `เ แ า` and other
  Lao vowels follow the same split; Hebrew and Arabic letters other than those probed."
- T-3325 (Out of scope / Context) leaves the other decimal-digit blocks unprobed: Bengali U+09E6-U+09EF,
  Lao U+0ED0-U+0ED9, Tibetan U+0F20-U+0F29, Myanmar U+1040-U+1049 (and Tamil U+0BE6-U+0BEF here).
- Depends on T-3324 and T-3325 so that the cells this probe adds against Hebrew, Hangul, CJK and the
  Arabic-Indic digits fail only for the new points, not for the rules those tasks implement.
- Ours: `element` in crates/rustsheet-engine/src/functions/util/collation.rs (after T-3324 places the probed
  blocks) and the shared number-text parser T-3325 extends.
- Questions, each with at least two cells:
  1. Block order of one or two letters of Bengali (`অ ক`), Tamil (`அ க`), Georgian (`ა ბ`), Armenian
     (`ա բ`, with `Ա` for case folding) and Ethiopic (`ሀ ለ`) against the ladder `"z"`, `"ω"`, `"я"`, `"ア"`,
     `"א"`, `"ا"`, `"क"`, `"ก"`, `"ກ"`, `"𝐀"`, `"가"`, `"一"`; code-point order inside each.
  2. Bengali and Tamil vowel signs and virama: letters (as Devanagari's) or accents.
  3. Digits: rank of `"3"` in each of Bengali, Tamil, Lao, Tibetan and Myanmar against `"3"`, `"٣"`, `"३"`,
     `"3a"`; `VALUE` of each digit alone, two digits, and with a decimal point (wrap in IFERROR and add
     `ERROR.TYPE` where it fails); COUNTIF `"="&x` over the digit column.
  4. `™`: `"™"="tm"`, `"™"` against `"tl"`, `"tm"`, `"tn"`, `"Ab"`, `"Az"`, `"b"`; XMATCH `"tm"` against `™`.
  5. `∞` against `"!"`, `"+"`, `">"`, `"←"`, `"✓"`, `"0"`, `"9"`, `"a"`.
  6. Astral: a Plane 1 symbol (`🀄` U+1F004), a Plane 2 ideograph other than `𠀀`, a Plane 14 tag character,
     and emoji sequences (`"👍"` vs `"👍🏽"` with a skin-tone modifier, `"👨‍👩‍👧"` ZWJ sequence, `"❤"` vs
     `"❤️"` with VS16) against `"😀"`, `"𝐀"`, `"가"` and each other; `=` between each sequence and its base.
  7. Thai vowels other than `เ แ า` (`โ ใ ไ` prevowels, `ะ ิ ี ุ`) and Lao `ແ ໂ ໃ ໄ`, `າ`: the same split as
     T-3276 found (Thai prevowels before the consonants, Lao after) or not.
  8. Hebrew final forms (`ך ם ן ף ץ` against their base letters) and Arabic letters with hamza (`أ إ آ`)
     against `ا`.
- Method: the `excel-probe` skill, excel-graph, laid out like probes/t3276-collation-other-scripts (Data
  column, `SUMPRODUCT(--(range<x))` ranks, `=` counts, chosen pairs, XMATCH and COUNTIF columns, the VALUE
  cells apart). Build strings with UNICHAR. Keep it under about 1,500 formula cells. Milestone tag M2
  (decision 0051).

## Acceptance criteria
- [ ] A probe case `probes/t3399-collation-open-scripts` (milestone M2) is committed with its excel-graph
      oracle sidecar and answers questions 1-8 above with at least two cells each.
- [ ] coercion.md states the result for each question with the case id and cells (VERIFIED, or UNVERIFIED
      with the competing readings where the cells do not settle it); the "Every other character ...
      UNVERIFIED" sentence and the T-3276 section's closing UNVERIFIED list are narrowed to what is still
      unprobed. A VALUE finding goes to the section T-3325 cites as well.
- [ ] Every cell the engine fails is a known failure in eval/baseline/probes.txt (`make eval-baseline`,
      0 regressions), and an engine task filed in the same PR (`make new-task`, milestone M2, lane engine)
      lists them (case stem and A1 names in one bullet, decision 0159), so `make exit-owners MILESTONE=M2`
      reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing collation.rs or the number parser (the engine task this files).
- Invisible format characters (soft hyphen, zero-width characters, bidi marks, BOM): T-3398.
- Kana and full-width forms (T-3381), the Latin, Greek and Cyrillic letters T-3305 probes, and case mapping.
