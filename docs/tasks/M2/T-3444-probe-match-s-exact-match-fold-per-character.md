---
id: T-3444
title: "Probe MATCH's exact-match fold per character where the t3298 sweep blocks disagree, and whether characters Excel's table lacks equal each other and empty text"
milestone: M2
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Excel evidence states, per character, which case pairs MATCH's exact match (type 0) treats as equal over the
characters of probes/t3298-case-mapping-sweep, and pins the rule that characters Excel's comparison table lacks
compare equal to one another.

## Context
- probes/t3298-case-mapping-sweep Calc!C1:D48 give, per block of 32 one-to-one case pairs (Data!D upper, Data!E
  lower), the first position MATCH finds for each lower among the block's uppers (C) and the reverse (D). Of 1514
  pairs, 982 match each other both ways, 374 match nothing, and 158 match another position.
- The 158 show a quirk lookup.md now states only as an observation: characters Excel's table seems not to know
  (Unicode additions after about 5.0 in the BMP) equal one another, like characters with no sort weight. For example
  Calc!C17 position 22: the Cherokee small letter U+AB70 matches U+10C7 (position 20), and Calc!C15 position 4:
  U+0529 matches U+0524. Supplementary-plane letters never fold and never match one another (C38 and D38 from position 24, and C39:D48,
  are all 0). Calc!C37 and D37 (the end of Latin Ext-D) are the densest block of such matches.
- Questions for one-cell-per-question probes (build the text with UNICHAR):
  - Does `MATCH(UNICHAR(x),{"",...},0)` or `UNICHAR(x)=""` find a weightless character equal to empty text? Is
    "a"&UNICHAR(x) equal to "a"? Do two different weightless characters equal each other in `=`, EXACT, COUNTIF,
    XMATCH(..,0) and VLOOKUP(..,FALSE)?
  - Which BMP characters are weightless: probe one representative of every block the sweep shows "other", then
    sweep the block edges.
  - The per-character fold of the "other" pairs in Greek (ϐ ϑ ϕ ϖ ϰ ϱ ϵ µ ι ͅ), Cyrillic Ext-C (U+1C80-U+1C88),
    Latin (ı ſ ǅ ǈ ǋ ǲ ẛ), and Georgian Mtavruli (U+1C90-U+1CBF).
- lookup.md "MATCH, VLOOKUP and HLOOKUP exact-match keys fold case one character to one" holds the current text.

## Acceptance criteria
- [ ] A probes case `probes/t3444-*` with excel-graph sidecars answers the questions above.
- [ ] lookup.md (and coercion.md if `=` is affected) states the weightless-character rule and the per-block fold
      table precisely enough for T-3445 to implement it, citing cells.
- [ ] Failing cells are known failures owned by T-3445 (or a new task); `make exit-owners MILESTONE=M2` reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the fold (T-3445). UPPER and LOWER (T-3443).
