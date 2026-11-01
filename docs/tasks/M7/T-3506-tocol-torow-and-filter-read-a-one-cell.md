---
id: T-3506
title: "TOCOL, TOROW and FILTER read a one-cell error reference in an array argument as a 1x1 array (gen-m7 arg-pairs)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
TOCOL, TOROW and FILTER read a one-cell reference to an error cell, passed as an array argument (TOCOL's and TOROW's `array`, FILTER's `array` and `include`), as a 1x1 array holding the error, the way Excel does. They no longer return that error before checking their other arguments.

## Context
- Found by the T-3497 arg-pairs batch (gen-m7, excel-graph, 2026-10-31). On the Data sheet of these cases G1 is 7, G2 0, G3 -2.5, G4 1.5, G5 the text "12", G6 "x", G7 an empty text, G8 FALSE, G9 `#DIV/0!` (a constant error cell), and H1 and H2 are blank.
- Excel, by the oracle sidecars:
  - `TOROW(Data!G9,Data!G1)` is `#VALUE!`: ignore 7 is invalid.
  - `TOCOL(Data!G9,3,Data!G2)` is `#CALC!`: ignore 3 drops the error, so nothing is left.
  - `TOCOL(Data!G9,2,Data!C3:D4)` spills a 2x2 of `#VALUE!`/`#CALC!`, lifted over scan_by.
  - `FILTER(Data!G9,Data!G2)` is `#CALC!`: the include is FALSE, so the result is empty.
  - `FILTER(Data!G9,(Data!B1:B12>(-1)),Data!G3)` is `#VALUE!`: a 1x1 array against a 12-row include.
  - `FILTER(Data!A1:D12,Data!G9,Data!G3)` is `#VALUE!`: a 1x1 include against 12 rows.
  - In every one of these we return `#DIV/0!`, the referenced cell's error.
- The error survives only where it would be an element of the result, for example TOCOL with ignore 0 or 1.
- Quirk recorded in docs/excel-semantics/arrays.md, "A one-cell error reference is a 1x1 array in TOCOL, TOROW and FILTER (T-3497)".
- Cells (57; all owned here):
- gen-m7/20df23f84a528470: A8724, A8968, A9212, A9334, A9456, A9944
- gen-m7/470d4f8391f9fe15: A8724, A8968, A9090, A9212, A9334, A9456
- gen-m7/68e7f10771a7118a: A10798, A11164, A11286, A11408, A11530, A11774, A11896
- gen-m7/7bd03be97c715dae: A2014
- gen-m7/7cd94f345acd55bb: A8480, A8724, A8846, A8968, A9090, A9212, A9578, A9579, B9578, B9579
- gen-m7/b24463215bcb1ab9: A8846, A9090, A9334, A9456, A9578, A10066, B10066, B10067
- gen-m7/d7a127bd9334c2b4: A2380, A2502, A2624, A2868, A2990, A3112, A3234, A3356
- gen-m7/ee81c689b5fde629: A794, A916, A1038, A1160, A1282, A1404, A1526, A1648, A1770, A1892, A2014

## Visible-parity stage
Decision 0083: every cell listed under Context is visibly off (a wrong error type, or an error where Excel spills), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text; the acceptance criteria state it.

## Acceptance criteria
- [x] Every cell listed under Context matches Excel (`make eval-show` on each case), with unit tests for the TOCOL/TOROW ignore paths and the FILTER shape check over a one-cell error reference.
- [x] Their lines leave eval/baseline/gen-m7.txt (`make eval-baseline`, 0 regressions); the steward lowers the visible-parity lock by the same count in its daily `make eval-lower-visible-lock` run.
- [x] `make check` green; reviewers approve.

## Out of scope
- The argument-check order of TOCOL/TOROW/SORT (T-3507) and SORT's lifted by_col (T-3508).

## Outcome
An error read through a one-cell reference is now a 1x1 array holding it for TOCOL's and TOROW's `array` and FILTER's
`array` and `include` (`one_cell_error_ref_as_array`, functions/spill_functions.rs), so the other arguments are checked
first: an invalid ignore is `#VALUE!`, ignore 2/3 leave `#CALC!`, a range ignore lifts per element, and FILTER's shape
check sees a 1x1 array. FILTER checks a one-cell reference include's shape before its elements (`include_axis`). A literal
error is still the error, as Excel has it (gen-m7/20df23f84a528470 A8907, ee81c689b5fde629 A977); a wrongly shaped
multi-cell include holding an error keeps its old order (unprobed). Unit tests:
`tocol_torow_one_cell_error_reference_is_a_one_by_one_array`, `filter_one_cell_error_reference_is_a_one_by_one_array`.
arrays.md "A one-cell error reference is a 1x1 array in TOCOL, TOROW and FILTER" is marked implemented.
Eval delta: all 57 owned cells pass (all 8 cases pass in full); eval/baseline/gen-m7.txt -57 lines, 0 regressions.
gen-m7 visibly off 109 -> 52 of 50,476 scored cells, allowance 50: M7's gen row is still not met (52/50). The other 52
are T-3507's 25 and T-3508's 27; `scripts/exit_owners.py --milestone M7` finds 0 unowned. The visible-parity lock is
lowered by the steward's daily run (`make eval-lower-visible-lock`), not in this PR (2744 visibly off against 2830
allowed after this change).
Review round 1: FILTER's shape-first return now keys on `wrap_one_cell_error_ref` reporting that it wrapped (no
behaviour change: a `ReferenceKind` is one cell, so a multi-cell include never reached it; pinned by
`filter_wrongly_shaped_range_include_with_an_error_is_the_error`). Follow-up: T-3667 (do other array functions read a
one-cell error reference the same way).
