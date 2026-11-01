---
id: T-3507
title: "TOCOL, TOROW and SORT argument-check order: which invalid argument beats an error in a later one (gen-m7 arg-pairs)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
TOCOL, TOROW and SORT choose the same error as Excel when one argument is invalid and a later argument is an error.

## Context
- Found by the T-3497 arg-pairs batch (gen-m7, excel-graph, 2026-10-31). On the Data sheet of these cases G1 is 7, G2 0, G3 -2.5, G4 1.5, G5 the text "12", G6 "x", G7 an empty text, G8 FALSE, G9 `#DIV/0!` (a constant error cell), and H1 and H2 are blank.
- TOCOL and TOROW: a text `ignore` ("abc", "x", "") beats an error in `scan_by_column` with `#VALUE!`, and we return the scan_by error. Examples: `TOROW(Data!A7:D7,"abc",#NULL!)`, `TOCOL(Data!C5,Data!G6,Data!G9)`, `TOCOL(Data!B3:C3,"",#NAME?)`.
  - An out-of-range *number* (7, -2.5) loses to the scan_by error. Our answer already matches there, in passing cells such as 7cd94f345acd55bb A11530 and 1157e6d3a3c6ae12 A1160: keep them passing.
  - A range ignore lifts per element: 52ccea5df17b29a8 A10676:B10676 is `#VALUE!` then `#DIV/0!`, and B10616 is the only `#VALUE!` of A10615:B10616.
- SORT: a *blank* `sort_index` or `sort_order` beats a later error with `#VALUE!`, and we return the error. Examples: `SORT(0.3,Data!H1,-1,#NAME?)`, `SORT(Data!A4:B6,2,Data!H2,Data!G9)`, `SORT(Data!B3:D8,Data!A3,#DIV/0!,#NUM!)` with A3 blank.
  - A text or FALSE sort_index loses to a later error, and we return `#VALUE!`. Examples: `SORT(Data!C4,"",Data!B10:D12,#DIV/0!)` is `#DIV/0!`, and `SORT({#DIV/0!;2;#VALUE!},"abc",#NULL!)` is `#NULL!`.
  - Passing counterexamples to keep: `SORT(0.5,Data!G8,Data!G9)` (91b6b88061b805b1 A2990) and 75534bc7e1ce6474 A11652, both `#DIV/0!`.
- Opus: the order has to be worked out from these cells (and, where they don't settle it, new excel-graph probes via the `excel-probe` skill). It is not one left-to-right rule.
- Quirk recorded in docs/excel-semantics/arrays.md, "TOCOL, TOROW and SORT: which bad argument wins (T-3497)".
- Cells (27; all owned here):
- gen-m7/1157e6d3a3c6ae12: A4393, A4454, A5735, A5796
- gen-m7/52ccea5df17b29a8: A4393, A4454, A5613, A5674, A10676, B10616
- gen-m7/75534bc7e1ce6474: A245, A306, A1526, A4027, A4088
- gen-m7/8217549f0b417328: A611, A1709, A1770
- gen-m7/91b6b88061b805b1: A367, A428, A1709, A1770, A5308, A11103, A11164, A11835
- gen-m7/d2945e939d6fe742: A7138

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong error type), except 52ccea5df17b29a8 A5613 and 75534bc7e1ce6474 A4027, which score `unsupported` (ours is `#NAME?`) and so are not in the visible-parity count. This task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the acceptance criteria state it.

## Acceptance criteria
- [x] Every cell listed under Context matches Excel; the rule is written into the arrays.md entry with the cells that fix it, and unit tests cover each branch.
- [x] Their lines leave eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions).
- [x] `make check` green; reviewers approve.

## Out of scope
- A one-cell error reference as the array (T-3506) and SORT's lifted by_col (T-3508).

## Outcome
- TOCOL and TOROW (`lift_flatten`): the ignore is cast to a number before the scan_by_column error is read, so a text ignore
  is `#VALUE!` first; its range check (0 to 3) still comes after.
- SORT (`sort_number_list`, `fn_sort`): a one-value sort_index or sort_order that is an error stays an error, a blank cell is
  `#VALUE!`, and text reads as NaN, which fails validation after by_col, as a 2-D list already did. The array, sort_index
  and sort_order errors are now each lifted call's result, ahead of by_col's: that keeps a `#VALUE!` sort_order spreading
  over a lifted by_col (8217549f0b417328 A855:B856, which this reordering would otherwise have broken) and fixes every
  T-3508 cell too, so T-3508 is done here.
- Unit tests: test_fn_tocol_torow.rs `text_ignore_beats_a_scan_by_error`; test_fn_sort_lift.rs
  `a_blank_key_beats_a_later_error_and_text_loses`, `an_argument_error_fills_the_lifted_shape`.
- Eval delta: gen-m7 known failures 111 -> 57 (54 fixed: these 27 and T-3508's 27; 0 regressions), pass 99.78% -> 99.89%.
  The 57 left are T-3506's. Visible-parity count 2763 in-scope cells (lock 2830, not lowered here).
- arrays.md "TOCOL, TOROW and SORT: which bad argument wins" and "SORT's lifted by_col spreads an argument error" updated.
