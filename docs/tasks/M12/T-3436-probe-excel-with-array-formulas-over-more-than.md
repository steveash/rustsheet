---
id: T-3436
title: "Probe Excel with array formulas over more than 2^20 cells, entered and in a file"
milestone: M12
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Know what Microsoft 365 Excel does with an array formula larger than 2^20 cells, so that two DEVIATION entries can cite
evidence: one for entering such a formula, one for opening a file that holds one.

## Context
T-2896 and T-3426 refuse array areas above 2^20 cells (one whole column), for two directive-5 reasons: evaluation
fills a CSE anchor's whole range, and the clears walk it.
- `set_user_array_formula` refuses a CSE formula entered over more than 2^20 cells (docs/excel-semantics/arrays.md,
  "A legacy array formula entered over more than 1,048,576 cells is refused").
- Loading refuses a file whose CSE or cached spill range is above 2^20 cells, and one whose anchors claim more cells
  than it stores (docs/excel-semantics/xlsx-storage.md, "A workbook whose indices dangle is refused outright").

What Excel does is UNVERIFIED in both cases. Use the `excel-probe` skill (excel-graph). Probe:
- a CSE formula entered over A:B (2^21 cells);
- an xlsx whose `<f t="array" ref="A1:B1048576">` holds every cell;
- the same file with the spill cells left out;
- the `ref` Excel writes for a dynamic array anchor that shows #SPILL! (the engine resets its range to the anchor, and
  the load check assumes Excel writes no larger range).

## Acceptance criteria
- [ ] Each probe is recorded, and arrays.md and xlsx-storage.md cite the result in their Status lines (the refusal
      stays a DEVIATION if Excel accepts).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
