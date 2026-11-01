---
id: T-3657
title: "PROPER over non-ASCII text: word starts and case mapping beyond the probed characters"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3298]
---

## Goal
PROPER gives Excel's text over non-ASCII text: which characters start a word, and the title or upper and lower mapping of each cased character, across the scripts and blocks Excel case-maps.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md, branch `claude/sharp-mendel-13x2km`). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports some non-ASCII case mappings of PROPER and LOWER differing from Excel's. LOWER
(and UPPER) over every cased character is T-3298's sweep; PROPER adds two rules of its own that T-3298 does not
cover: the word-start test (text.md "CLEAN character set and PROPER word starts and case mapping (T-1307)": letters
after digits, apostrophes, non-ASCII letters) and the mapping of a word start (titlecase digraphs such as Ǆ/ǅ/ǆ,
ǈ, ǋ, ǲ versus upper case). Done: T-1307, T-1747 (expanding mappings, final sigma), T-1587 (lone surrogates). Sweep
word starts after each non-letter class (combining marks, modifier letters, non-cased letters such as CJK and
Hebrew, digits of other scripts, ZWJ/ZWNJ, punctuation), and the word-start mapping of each titlecase-sensitive
character, using T-3298's case table once it lands. Code: `fn_proper` in functions/text/string_format.rs.

## Acceptance criteria
- [ ] A probe case (excel-graph) covers the word-start classes and titlecase characters above.
- [ ] The engine matches every probed cell; text.md states PROPER's word-start rule and mapping.
- [ ] Every quirk found is written up in docs/excel-semantics/text.md in the same PR, with its probe cells cited.
- [ ] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- LOWER and UPPER's table (T-3298).
