---
id: T-3507
title: "TOCOL, TOROW and SORT argument-check order: which invalid argument beats an error in a later one (gen-m7 arg-pairs)"
milestone: M7
lane: functions-lookup
status: todo
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
- TOCOL and TOROW: an invalid `ignore` beats an error in `scan_by_column`, so the result is `#VALUE!`. Examples: `TOROW(Data!A7:D7,"abc",#NULL!)`, `TOCOL(Data!C5,Data!G6,Data!G9)`, `TOCOL(Data!B3:C3,"",#NAME?)`, and a range ignore `TOCOL(Data!D$11,Data!C3:D4,Data!G9)`. We return the scan_by error.
- SORT is mixed:
  - A text `sort_index` ("", "abc", "x") loses to an error in a later argument: `SORT(Data!C4,"",Data!B10:D12,#DIV/0!)` is `#DIV/0!` and `SORT({#DIV/0!;2;#VALUE!},"abc",#NULL!)` is `#NULL!`. We return `#VALUE!`.
  - A blank sort_index or sort_order wins with `#VALUE!` over a later error: `SORT(0.3,Data!H1,-1,#NAME?)` and `SORT(Data!A4:B6,2,Data!H2,Data!G9)`.
  - So do an out-of-range index and a FALSE index: `SORT(Data!B3:D8,Data!A3,#DIV/0!,#NUM!)` and `SORT(Data!D7:E12,Data!G8,Data!E7,Data!G9)`. In these we return the error.
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
Decision 0083: every cell listed under Context is visibly off (a wrong error type, or an error where Excel spills), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the acceptance criteria state it.

## Acceptance criteria
- [ ] Every cell listed under Context matches Excel; the rule is written into the arrays.md entry with the cells that fix it, and unit tests cover each branch.
- [ ] Their lines leave eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- A one-cell error reference as the array (T-3506) and SORT's lifted by_col (T-3508).
