---
id: T-3398
title: "Probe how Excel compares texts holding ignorable format characters: soft hyphen, zero-width space and joiners, bidi marks, BOM, word joiner, variation selector and the C1 controls"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An excel-graph probe case shows how Excel's text comparison (`=`, `<`, `>`), the text criteria of
COUNTIF/SUMIF/COUNTIFS, and the exact lookups treat the invisible format characters that pasted web
and PDF text carries, and docs/excel-semantics/coercion.md states the rule for each, so the engine's
current treatment is either VERIFIED or owned by an engine task.

## Context
- No collation probe holds any of these characters today (`rg` over coercion.md, criteria.md and
  lookup.md finds no soft hyphen, U+200B-U+200F, U+2060, U+FEFF or U+FE0F). The Latin-1 probe of
  T-1469/T-1595 (coercion.md "Collation equality in lookups and criteria; unprobed characters (T-1590)")
  ranked `¡ ¦ ¨ ¯ ´ ¸ ¿` and the Latin-1 symbols but not the soft hyphen (CHAR(173)), and ASCII control
  characters are VERIFIED as skipped (tiebreak only, T-1662) while the C1 controls U+0080-U+009F are not
  probed. CLEAN keeps U+00AD, U+200B and U+FEFF (text.md "CLEAN character set ... (T-1307)"), so such
  text reaches comparisons in real workbooks.
- Ours: `element` in crates/rustsheet-engine/src/functions/util/collation.rs sends every one of these
  characters to `other(c)`, a primary weight after every letter. So today `"a"&UNICHAR(8203)&"b"` sorts
  after `"az"` and is not equal to `"ab"`; if Excel's (Windows NLS) table makes them ignorable, as it
  does the hyphen and the ASCII controls, every `=`, criteria and XMATCH cell holding them is wrong.
  Exact MATCH/VLOOKUP/HLOOKUP fold through `folded_units` (functions/criteria.rs) and
  `fold_exact_lookup` (collation.rs), which keep every UTF-16 unit; wildcard criteria go through
  `wildcard_text_matcher` (criteria.rs), where `?` counts characters.
- Characters to cover (build each with UNICHAR so the workbook stays ASCII): U+00AD soft hyphen,
  U+200B zero-width space, U+200C ZWNJ, U+200D ZWJ, U+200E LRM, U+200F RLM, U+2060 word joiner,
  U+FEFF BOM/ZWNBSP, U+FE0F variation selector 16, U+0085 NEL and U+0081/U+009F (C1 controls).
- Questions, each answered by at least two cells per character:
  1. Equality: `"a"&c&"b"="ab"`, `c=""` (the character alone against empty text), `"ab"&c="ab"`.
  2. Order when not equal: rank against the ladder `""`, `"a"`, `"ab"`, `"a-b"`, `"a'b"`,
     `"a"&CHAR(1)&"b"`, `"a b"`, `"ac"`, `"az"`, `"α"` (is it skipped with a tiebreak like the hyphen
     and CHAR(1), a punctuation weight, or a letter after `z` as ours?), and the tiebreak order among the
     characters themselves (`"a"&c1&"b"` vs `"a"&c2&"b"`).
  3. Criteria: `COUNTIF(r,"ab")`, `COUNTIF(r,"<>ab")`, `COUNTIF(r,"a?b")`, `COUNTIF(r,"a*b")` over a Data
     column holding `"ab"` and each `"a"&c&"b"` (does `?` match the invisible character?).
  4. Lookups: `MATCH("ab",r,0)` and `XMATCH("ab",r,0)` over that column (collation equality applies to
     XMATCH and not to MATCH for `æ`/`ae`; check whether the same split holds here), and
     `EXACT("ab","a"&c&"b")` as the control.
  5. LEN of each `"a"&c&"b"` (UTF-16 count, expected 3) so a lost character in the oracle path is visible.
- Method: the `excel-probe` skill, excel-graph, laid out like probes/t3276-collation-other-scripts (a Data
  column of strings, `SUMPRODUCT(--(range<x))` ranks and `=` counts, then chosen pairs and the criteria
  and lookup columns). Keep it under about 600 formula cells. Milestone tag M2 (decision 0051).

## Acceptance criteria
- [ ] A probe case `probes/t3398-collation-ignorables` (milestone M2) is committed with its excel-graph
      oracle sidecar and answers questions 1-5 for every character listed above.
- [ ] coercion.md gains a section stating, per character or per group of characters that behave alike,
      the observed equality, order, criteria and lookup behaviour with the case id and cells (VERIFIED),
      and criteria.md/lookup.md point to it where criteria or exact lookups differ from `=`.
- [ ] Every cell the engine fails is a known failure in eval/baseline/probes.txt (`make eval-baseline`,
      0 regressions), and an engine task filed in the same PR (`make new-task`, milestone M2, lane engine)
      lists them (case stem and A1 names in one bullet, decision 0159), so `make exit-owners MILESTONE=M2`
      reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing collation.rs or criteria.rs (the engine task this files).
- Emoji sequences (skin-tone modifiers, ZWJ emoji, VS16 after an emoji) and other scripts: T-3399.
- TRIM, CLEAN and SUBSTITUTE of these characters (text functions, not comparison).
