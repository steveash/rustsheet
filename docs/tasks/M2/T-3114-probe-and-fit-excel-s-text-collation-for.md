---
id: T-3114
title: "Probe and fit Excel's text collation for letters beyond Latin-1 (Latin Extended-A, combining marks, accented Greek, Cyrillic)"
milestone: M2
lane: engine
status: blocked
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Text comparison (`<`, `>`, `=` between texts, the ordered and `=`/`<>` text criteria of COUNTIF/SUMIF/SUMIFS/COUNTIFS,
SORT, XMATCH/XLOOKUP exact match) orders and equates the common letters outside ASCII and Latin-1 the way Excel does,
backed by a committed oracle case. Today every such letter sorts after `z` by code point.

## Context
- `cmp_text_excel` and `collation_key` (crates/rustsheet-engine/src/functions/util/collation.rs) place only the
  characters probed by T-1469, T-1590, T-1595 and T-1662. Its module docs say: "Every other character keeps code-point
  order after the letters, as before T-0220." `element` sends any letter it does not list to
  `OTHER_BASE + code point`. So `"čapek" < "dvorak"`, `"ş" < "t"` and `"ő" < "p"` are all FALSE in the engine, and
  `"a"&UNICHAR(769)` (a combining acute) never equals `"á"`. docs/excel-semantics/coercion.md "Text comparison uses
  Excel's collation" also says (fitted key, item 3): "Characters outside ASCII and the Latin-1 letters ... are
  UNVERIFIED: they keep code-point order after the letters", and "Tilde and cedilla are placed by Windows' sort-key
  order, UNVERIFIED".
- Probes already exist for Latin-1, `œ`/`þ`/`ð`/`ł`/`đ`/`ı`, the Latin-1 symbols and `α < β < ω`, but each was
  probed as a single character with no accent:
  probes/text-collation-order, -unprobed, -longer, -lookups and -ifs (excel-graph, 2026-10-13..16). Nothing probes the
  Latin Extended-A letters of real names (Czech, Polish, Turkish, Hungarian, Romanian, Baltic). Nothing probes
  canonically equivalent sequences (base letter plus combining mark against the precomposed letter), Greek with tonos
  or dialytika, or Cyrillic (`ё` against `е`, upper case against lower case).
- Why opus: this is a fit. The probe must be designed so that it tells the candidate rules apart: a caron, ogonek,
  breve, macron, double acute and dot above, each as a later accent on its base letter, or as its own letter after the
  base. The fitted weights must also extend the three-pass key without breaking any cell of the five collation probes above, all of which pass today.
  Whether Excel for the web compares canonical equivalents as equal is an open question, and the answer decides how
  `element` reads combining marks (U+0300-U+036F).
- Method: rank strings with `SUMPRODUCT(--(range<x))` and test equality with `=` cells, as T-1469 did
  (`excel-probe` skill, excel-graph). Keep upper and lower case pairs (`Č`/`č`, `Ё`/`ё`, `Ά`/`ά`) so case folding
  through `fold` (`to_lowercase`) is checked as well. MATCH(...,0) and VLOOKUP(...,FALSE) do not use the collation
  (T-1590), so add a few exact MATCH cells over the new letters to confirm their case-folded exact key still holds.

## Acceptance criteria
- [ ] A new probe case `probes/t3114-collation-beyond-latin1` (oracle-filled by excel-graph) ranks at least: the Latin
      Extended-A letters `ā ă ą ć č ď ē ė ę ě ğ ģ ī į ķ ĺ ľ ń ņ ň ő ŕ ř ś ş š ţ ť ū ů ű ų ź ż ž` and the
      upper-case form of each, against their base letter and the next letter. It also ranks two-letter
      strings that tell an accent from a separate letter (`"ča"` against `"cb"` and `"da"`), base plus U+0301/U+030C
      against the precomposed form (`=` and `<`), `ά έ ή ί ό ύ ώ ϊ ΐ` against `α`…`ω`, and the Cyrillic alphabet with
      `ё`/`Ё` and `й`. It also has `=` cells between upper and lower case for the Greek and Cyrillic letters.
- [ ] Every cell of that case passes. The cells of probes/text-collation-order, -unprobed, -longer, -lookups and
      -ifs still pass, and so do the gen-m2 text-comparison and criteria cells (`make eval-check`: 0 regressions).
- [ ] coercion.md's collation section records the probed order as VERIFIED, with the case id and cells. Each rule the
      engine infers beyond the probed characters is named as UNVERIFIED. The stale sentence about `þ`, `ð`, `œ`,
      Greek and `§` keeping code-point order is corrected. The module docs of collation.rs say the same.
- [ ] `collation_key` and `cmp_text_excel` still agree (two texts are equal under one exactly when they are under the
      other). A unit test covers a new accented pair and a combining sequence through both.
- [ ] `make check` green; reviewers approve.

## Out of scope
- CJK, kana, Hangul, Arabic, Hebrew, Indic scripts and emoji ordering (file a follow-up probe task if the evidence
  suggests Excel gives them structure beyond code-point order).
- The approximate lookups' text order (T-3115) and the exact MATCH/VLOOKUP key beyond the confirming cells above.
- Case mapping in UPPER/LOWER/PROPER (text.md, T-1747).

## Outcome
- Probe `probes/t3114-collation-beyond-latin1` (excel-graph, 2026-10-29): 282 strings ranked, plus
  pairwise, exact MATCH and COUNTIF cells; 986 cells, all pass.
- Finding (coercion.md "Letters beyond Latin-1 and combining marks"): every Latin Extended-A, Greek
  accented and Cyrillic letter probed is its base letter plus an accent; a base letter plus a combining
  mark equals the precomposed letter; `Σ = σ = ς`, and exact MATCH of `ς` finds `σ`.
- Engine: `collation.rs` folds case, then decomposes canonically (`DECOMPOSED`) and gives a combining
  mark's accent to the letter before it; the accent order gains breve, dot above, caron, macron,
  ogonek and double acute. `criteria::fold_run` folds `ς` to `σ` for the exact, wildcard and
  D-function prefix matches on both sides.
- Eval delta: +1 probe case (986 cells, all passing), 0 regressions; the five earlier collation
  probes and t3115 still pass in full.
- Follow-up: T-3154 (accents across Latin-1 and Extended-A on one letter, stacked marks, `ё`/`й`).

## Blocked because
Blocked because: review-round limit
- Round 3 (excel-fidelity, REQUEST_CHANGES; security, architecture and quality had approved): one
  MAJOR finding, a coercion.md line said exact MATCH equates combining-mark spellings with precomposed
  letters, which Calc!H8/H9 contradict. It is fixed on this branch (doc only; the engine already
  matched all 986 cells), together with the A123:A139 citation and Latin Extended-B in T-3154.
  A re-review from round 1 (work-loop step 5c) is all that remains.
