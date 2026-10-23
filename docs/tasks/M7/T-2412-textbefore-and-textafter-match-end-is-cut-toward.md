---
id: T-2412
title: "TEXTBEFORE and TEXTAFTER: match_end is cut toward zero, so -0.0001 is 0 (gen-m7)"
milestone: M7
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
TEXTBEFORE and TEXTAFTER read match_end the way they read match_mode: cut toward zero, then 0 or 1, so a match_end of -0.0001 is 0
(not `#VALUE!`), as Excel gives.

## Context
Gap triage of the T-2254 gen-m7 sweep (excel-graph, 2026-10-22); 1 cell.
`TEXTAFTER(LOWER("'a"),Data!A11:B12,1,1,Data!E2)` with the delimiters 0.5, 1, 1.005, -0.0001 (none in "'a") and match_end E2 = -0.0001
is `#N/A` (2d9b48c6363e7a16 A660): match_end is read as 0, since with match_end 1 the end of the text would match and give "". Ours
`#VALUE!` (`TEXTAFTER("'a","zz",1,1,-0.0001)` too, checked 2026-10-22). text.md "TEXTBEFORE and TEXTAFTER: instance_num past the text, a
number if_not_found" already has match_mode "cut toward zero and must then be 0 or 1" (`TEXTBEFORE("abc","b",1,-0.5)` is "a"); this
cell extends it to match_end. Which other match_end values are refused (2, -1, 1.5, TRUE) is unprobed: keep ours as it is for those
unless the same helper decides them, and note it. Code: `around_options` in functions/text/before_after.rs. Quirk entry: that text.md entry
(sentence added by this triage).

Owned gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22): 1 known-failure lines in eval/baseline/gen-m7.txt, 1 formulas.
- gen-m7/2d9b48c6363e7a16.case.json A660 `=TEXTAFTER(LOWER("'a"),Data!A11:B12,1,1,Data!E2)`: A660 Excel #N/A, ours #VALUE!; cells (1): A660

## Acceptance criteria
- [x] Visible parity: this 1 known-failure line disappears from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new
      known-failure lines in any baseline.
- [x] Unit tests: match_end -0.0001 and 0.5 behave as 0; 1 still matches the text's end.
- [x] text.md's entry has an "Ours:" line naming this task.
- [x] `make check` green; reviewers approve.

## Out of scope
- TEXTSPLIT (T-2315).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
`around_options` cuts match_end toward zero (like match_mode), so -0.0001 and 0.5 are 0. Eval: 1 known-failure line removed from gen-m7, none added. text.md "Ours:" line updated; 1.5 now reads as 1 (unprobed, noted).
