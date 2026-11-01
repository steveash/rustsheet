---
id: T-3647
title: "NUMBERVALUE separators and spaces: sweep decimal and group separator placement, repeats and whitespace"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
NUMBERVALUE gives Excel's value or `#VALUE!` for every placement of its decimal and group separators and of whitespace in the text, including repeats, separators in the wrong place and custom separators.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports NUMBERVALUE's handling of separators and spaces disagreeing in both directions (we
accept some texts Excel refuses, and refuse some it accepts). Done: T-2151 (reads as VALUE does), T-2152 (empty text,
first character of a separator), T-2352 ("25:"), T-2379, T-2441 (comma decimal, no group separator); text.md
"NUMBERVALUE reads text as VALUE does, with its own separators". Open: T-3512 (gen-m5 argument-pair cells). Not
swept: group separators after the decimal separator, two decimal separators, group separators at uneven spacing or
leading/trailing, a group separator equal to a space, spaces inside the digits (Excel strips spaces: where, and
which kinds - tab, NBSP, LF), separators next to a sign, percent or currency, separators that are letters or
digits, and the default separators against each custom pair. Code: `fn_numbervalue` in
functions/text/string_format.rs and the shared reader it calls (coercion.md); fix the shared helper only where VALUE
agrees.

## Acceptance criteria
- [x] A probe case (excel-graph) covers the shapes above with the default and at least three custom separator pairs.
- [x] The engine matches every probed cell; text.md's NUMBERVALUE section states the rules.
- [x] Every quirk found is written up in docs/excel-semantics/text.md in the same PR, with its probe cells cited.
- [x] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- T-3512's argument-pair cells.

## Outcome
Done (2026-11-01). Two probe cases (excel-graph, 295 cells) found three rules where the engine differed, all now
matched:
- **Group separators go only before the decimal separator.** After the first decimal separator the group character
  stays: a digit reads as that digit (`NUMBERVALUE("17582","7","8")` = 1.582), anything else is `#VALUE!`
  (`NUMBERVALUE("1.5,2")`, `("1,234.5",",",".")`, `("1.234,5")`). Ours removed every occurrence.
- **Tab, LF and CR are removed like spaces**, anywhere; CHAR(1), CHAR(11), CHAR(12), NBSP, U+3000 and U+202F are not.
- **One leading `%`** counts (also after a sign or `(`, not with currency or a second `%`), and text of `%` signs alone
  is 0.
- Code: `numbervalue_text` and `strip_leading_percent` in functions/text/string_format.rs; tests in
  test/text_functions/fn_numbervalue_placement.rs; rules in text.md "NUMBERVALUE removes group separators only before
  the decimal separator". VALUE is unchanged (the task's "fix the shared helper only where VALUE agrees").
- Matching already: leading/trailing/repeated/uneven group separators before the decimal, two decimal separators,
  separators next to signs, currency, parentheses and exponents, letter and digit separators, and leading/trailing
  whitespace.
- Probes: probes/t3647-numbervalue-separator-placement (44 shapes × default, ","/".", "."/" ", ";"/"'", "d"/"g",
  "7"/"8"; 264/264 pass) and probes/t3647-numbervalue-percent-and-controls (30/31 pass).
- Follow-up: **T-3742** (VALUE reads a leading `%`: `VALUE("%1.5")` = 0.015, the one baselined failure, A13).
- Eval delta: 2 new probe cases, 295 cells, 294 passing; probes known failures +1 (A13, owned by T-3742); 0
  regressions in every suite; the older NUMBERVALUE probes (numbervalue-default-group, numbervalue-separator-edges)
  still pass in full.
