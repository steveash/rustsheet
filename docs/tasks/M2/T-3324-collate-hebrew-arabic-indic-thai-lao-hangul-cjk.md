---
id: T-3324
title: "Collate Hebrew, Arabic, Indic, Thai, Lao, Hangul, CJK, symbols and astral characters as Excel does (T-3276 probe)"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`cmp_text_excel` orders Hebrew, Arabic, Devanagari, Thai, Lao, Hangul, CJK, the symbols above `€` and
astral characters as Excel does, so `=`/`<`, the criteria and the XMATCH/XLOOKUP `CollationKey` pass
the collation cells of probes/t3276-collation-other-scripts.

## Context
- Rules and cells: docs/excel-semantics/coercion.md "Other scripts, their marks and digits, symbols
  and astral characters (T-3276)". In short:
  - Block order: Latin, Greek, Cyrillic, kana, Hebrew, Arabic, Devanagari, Thai, Lao, the astral
    `𝐀`/`😀`, Hangul, CJK, then `𠀀`. Code-point order inside each script.
  - Hebrew points, Arabic harakat and the Thai tone mark are accents. Devanagari matras and the
    virama are letters.
  - Thai `เ`/`แ` sort before the consonants; Lao `ເ` keeps its code-point place.
  - Other-script digits are their digit plus a tiebreak (ASCII < Arabic-Indic < Devanagari).
  - Symbols `← ☺ ♥ ✓` sort between `>` and `0`, `∞` between `9` and `a`, `™` between `Ab` and `ÿ`.
- Ours today: `element` in crates/rustsheet-engine/src/functions/util/collation.rs weighs these
  `OTHER_BASE + code point` and astral characters from `UNIT_BASE`; `punctuation` stops at `€`.
- Failing cells:
  - probes/t3276-collation-other-scripts.case.json Calc!A8:A96, Calc!A98:A99, Calc!G6, Calc!G16, Calc!G20:G23, Calc!G27:G28, Calc!G31:G32

## Acceptance criteria
- [ ] Every Calc!A, B, D and G cell of probes/t3276-collation-other-scripts passes, and no collation
      probe regresses (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Digits in VALUE and numeric criteria (T-3325).
- Kana and full-width forms (T-3321).
