---
id: T-3648
title: "ARABIC: sweep the numeral shapes Excel accepts (case, non-canonical forms, sign, length, non-Roman characters)"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
ARABIC accepts exactly the numeral texts Excel accepts, with Excel's value, and gives Excel's error for the rest.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports ARABIC rejecting some inputs Excel accepts. Done: T-2153 (blank is 0, outer spaces
and tabs trimmed), T-2159 (lifting), T-2200; numbers.md "ARABIC of a blank cell, and padded numerals". Not probed:
lowercase and mixed case; non-canonical additive and subtractive forms (IIII, VV, IC, IM, XM, VX, LC, DM, IIX,
MMMMM); repeats past three; a leading minus (-XIV) and plus; the empty and 255-character limits (Excel documents
255); inner spaces; Unicode Roman numeral characters (U+2160..U+2188); numeric text; and ROMAN's own outputs for forms
1-4 read back (ARABIC(ROMAN(n,k)) for every form). Code: `fn_arabic` in
functions/math_and_trigonometry/arabic_roman.rs.

## Acceptance criteria
- [x] A probe case (excel-graph) covers the shapes above, including ARABIC(ROMAN(n,k)) for k 0-4 over a range of n.
- [x] The engine matches every probed cell; numbers.md states the accepted grammar.
- [x] Every quirk found is written up in docs/excel-semantics/numbers.md in the same PR, with its probe cells cited.
- [x] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- ROMAN.

## Outcome
Probed 217 cells (probes/t3648-arabic-numeral-shapes, excel-graph, 2026-11-01). Excel's ARABIC has no numeral
grammar: an optional leading `-`, then ASCII IVXLCDM letters in either case, summed right to left (a letter below the
largest seen so far is subtracted). A lone `-` is `#NUM!`; any other character, a `+`, numeric text and the Unicode Roman
numerals are `#VALUE!`; the text may be 255 characters before trimming. Every ROMAN form reads back. `from_roman` is
rewritten to that rule (the old classic-grammar parser rejected most non-canonical and every signed numeral), with unit
tests; numbers.md "ARABIC reads any Roman-letter string, right to left" records the grammar.
Eval delta: the new probe case passes 217/217 cells; no regressions.
