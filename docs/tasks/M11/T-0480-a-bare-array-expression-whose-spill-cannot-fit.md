---
id: T-0480
title: "A bare array expression whose spill cannot fit is #SPILL! before the array-size cap's #ERROR! (corpus-bench/55049, 21 cells)"
milestone: M11
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
A cell holding a bare array expression whose result cannot spill (its shape runs past the grid from the
anchor, or the spill area is occupied) is `#SPILL!`, as Excel has it, even when the array is larger than
the engine's 1,048,576-cell bound: the spill is refused from the shape, before anything is built, so the
directive-5 bound still holds.

## Context
Gap triage 2026-10-26 (external suites fetched: calipers, corpus-bench, poi). `corpus-bench/55049/1_55049_input.xlsx`
is a whole-case known failure (`F corpus-bench/55049/1_55049_input.xlsx *` in eval/baseline/corpus-bench.txt,
21 of 21 scored cells fail) and no open task owns it. Sheet `book1`, B3:B23, each
`=SUMPRODUCT(book2!A:A=book1!A3)*book2!H:J`: the left operand is a scalar, the right a whole-column
3-column range, so the broadcast result is 1,048,576 x 3 = 3,145,728 cells. Excel gives `#SPILL!` in every
cell (oracle: excel-graph sidecar, eval/suites/corpus-bench/oracle); ours gives `#ERROR!`:

- `book1!B3` .. `book1!B23` (21 cells): `=SUMPRODUCT(book2!A:A=book1!A3)*book2!H:J`, Excel `#SPILL!`, ours `#ERROR!`
  (`make eval-show CASE=corpus-bench/55049/1_55049_input.xlsx`).

Excel's rule is already recorded: docs/excel-semantics/arrays.md "Array results above 1,048,576 cells are
#ERROR!" states that Excel builds an array of a range's size however large and that a bare `=ABS(A:A)` or
`=DATE(11:1899,12,1)` in a cell is `#SPILL!` ("no room"), while RustSheet answers `#ERROR!` for any array
above one whole column (`limits::MAX_SIZE`, `check_array_size`, `broadcast_arrays` in
crates/rustsheet-engine/src/arithmetic.rs) — a DEVIATION under decision 0014, because the engine must not
allocate 3M cells of untrusted input. The deviation is only needed where Excel returns a **value** from such
an array (`ROWS(ABS(A:A))` = 1048576, `SUM(ABS(A:A))`); in a spill context Excel's answer is an error that
follows from the shape alone, so parity costs no allocation: the broadcast shape of the two operands
(rows = max, columns = max) decides `#SPILL!` before any cell is read.
Code: `Model::spill_dynamic_array` (crates/rustsheet-engine/src/model.rs, around line 970) already answers
`#SPILL!` with "Spill would exceed worksheet bounds", but only once an array value exists; the cap fires
first, inside `broadcast_arrays`/`check_array_size`. The fix belongs where a cell's formula result is about
to spill: compute the result shape (the broadcast of the operand shapes, a range's own shape, a lifted
call's broadcast) and refuse the spill from it. Keep every non-spill context (ROWS/COLUMNS/SUM over the same
expression) on today's path: those stay `#ERROR!` under the deviation, which the same arrays.md entry records.
This is an evaluator change across the operator broadcast, the lifting driver and the spill path, so it needs
a design decision about where the shape is computed; hence opus, and a short spec if it grows past M.

## Acceptance criteria
- [ ] `book1!B3:B23` of corpus-bench/55049 are `#SPILL!`, so the `F corpus-bench/55049/1_55049_input.xlsx *`
      line disappears from eval/baseline/corpus-bench.txt, with no new F line in any baseline
      (`make eval`, `make eval-baseline`).
- [ ] Engine tests: a bare `=A:A*2`-shaped expression over a whole column times a 3-column range in a cell is
      `#SPILL!`; `=ABS(A:A)` in a cell with an occupied neighbour is `#SPILL!`; `ROWS(ABS(A:A))` and
      `SUM(ABS(A:A))` keep their current results (the decision-0014 deviation is unchanged for value contexts);
      no allocation above `limits::MAX_SIZE` happens on any of these paths.
- [ ] docs/excel-semantics/arrays.md "Array results above 1,048,576 cells are #ERROR!" records the spill case
      with the corpus-bench/55049 cells and states what the deviation still covers.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lifting or kernel work budgets that answer `#ERROR!` for arrays that *do* fit (T-2795, T-2607).
- Making a value context (`ROWS`, `SUM`, `INDEX`) over an above-bound array return Excel's value: that is the
  standing deviation (decision 0014).
- The other unowned corpus-bench cases (T-0481, T-0482).

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong error (none is a last-digit difference), so the task is
in the campaign at the `semantics` stage. Its target is that every cell it owns has Excel's error value; the
first acceptance criterion states it.
