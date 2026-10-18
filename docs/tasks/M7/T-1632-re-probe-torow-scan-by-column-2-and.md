---
id: T-1632
title: "Re-probe TOROW scan_by_column 2 and boolean ignore, WRAPROWS boolean wrap_count and ISOMITTED of a blank reference"
milestone: M7
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for `TOROW(A1:C3,0,2)` (a scan_by_column of 2) and `WRAPROWS(B5:H5,TRUE)` that tests the
function and not the sheet layout.

## Context
Found in review of T-1627: probes/dynamic-array-reshape-and-isomitted B41 and C44 are `#SPILL!` in Excel
only because their results run into neighbouring cells (C41/D41, C45), so they say nothing about how
the functions treat those arguments. Add a new probe case (do not edit the existing one; decision 0043)
that wraps both forms in TEXTJOIN and ROWS/COLUMNS, fill it with the oracle, and record the result
in docs/excel-semantics/arrays.md ("WRAPROWS and WRAPCOLS" section).
Also probe the forms T-1627's cells could not tell apart: `COLUMNS(TOROW(A1:C3,TRUE))` (which ignore mode a
boolean selects), `COLUMNS(TOROW(A1:C1,1))` (ignore 1 dropping a blank, C1 blank), and
`LAMBDA(x,[y],ISOMITTED(y))(1,C1)` with C1 blank (a reference to a blank cell).

## Acceptance criteria
- [x] A new probe case with excel-graph values for every form above, shape-stable (TEXTJOIN, ROWS, COLUMNS).
- [x] The finding recorded in docs/excel-semantics/arrays.md; a fix task filed for any mismatch.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine fixes (filed as tasks).

## Outcome
New probe `probes/dynamic-array-reshape-reprobe` (excel-graph filled): all 20 cells already match ours, so no fix task. Findings in docs/excel-semantics/arrays.md (TRUE ignore is mode 1, scan_by_column 2 is truthy, WRAPROWS TRUE is 7x1, ISOMITTED of a blank reference is FALSE). Eval delta: +20 passing probe cells, no regressions.
