---
id: T-3657
title: "PROPER over non-ASCII text: word starts and case mapping beyond the probed characters"
milestone: M5
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: [T-3298]
---

## Goal
PROPER gives Excel's text over non-ASCII text: which characters start a word, and the title or upper and lower mapping of each cased character, across the scripts and blocks Excel case-maps.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports some non-ASCII case mappings of PROPER and LOWER differing from Excel's. LOWER
(and UPPER) over every cased character is T-3298's sweep; PROPER adds two rules of its own that T-3298 does not
cover: the word-start test (text.md "CLEAN character set and PROPER word starts and case mapping (T-1307)": letters
after digits, apostrophes, non-ASCII letters) and the mapping of a word start (titlecase digraphs such as Ǆ/ǅ/ǆ,
ǈ, ǋ, ǲ versus upper case). Done: T-1307, T-1747 (expanding mappings, final sigma), T-1587 (lone surrogates). Sweep
word starts after each non-letter class (combining marks, modifier letters, non-cased letters such as CJK and
Hebrew, digits of other scripts, ZWJ/ZWNJ, punctuation), and the word-start mapping of each titlecase-sensitive
character, using T-3298's case table once it lands. Code: `fn_proper` in functions/text/string_format.rs.

## Acceptance criteria
- [x] A probe case (excel-graph) covers the word-start classes and titlecase characters above.
- [x] The engine matches every probed cell; text.md states PROPER's word-start rule and mapping.
- [x] Every quirk found is written up in docs/excel-semantics/text.md in the same PR, with its probe cells cited.
- [x] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- LOWER and UPPER's table (T-3298).

## Outcome
- Probes (excel-graph, 2026-11-01): probes/t3657-proper-word-starts (177 cells: word starts after 105 characters
  of every non-letter class the task names, and the word-start, mid-word and after-digit mapping of 24
  titlecase-sensitive characters) and probes/t3657-proper-letter-sweep (one digit per BMP code point, 63,454 in all,
  saying whether PROPER reads it as a letter).
- Finding: PROPER's letters are Excel's own table (47,717 BMP code points in 328 ranges), not Unicode's Alphabetic
  property: most combining marks Unicode calls Alphabetic are not letters (Thai and two Mongolian marks are), 1,321
  newer Unicode letters are not letters, nor is any supplementary character; and PROPER lowers every non-letter
  (`PROPER("Ⓐx")` is `ⓐX`). Written up in text.md "PROPER word starts: Excel's own letter table (T-3657)".
- Engine: `is_proper_letter` (new functions/text/proper_letters.rs, the decoded table) replaces `is_alphabetic` in
  `proper_case`, which now lowers non-letters.
- Eval delta: the two new cases add 181 cells; 174 pass. 7 known failures (t3657-proper-word-starts S!B24, C14, C16,
  C18, C20, C23, D24) need Excel's old case table, which T-3583 (#2801) implements. No other case changed.
