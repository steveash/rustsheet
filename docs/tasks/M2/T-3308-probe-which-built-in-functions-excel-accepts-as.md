---
id: T-3308
title: "Probe which built-in functions Excel accepts as an intersection operand, to replace the engine's inferred refuse-by-default list"
milestone: M2
lane: research
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Excel evidence decides, for every call shape the engine's intersection parser treats by inference,
whether Excel for the web loads `=A1:A3 F(...)` (and `=F(...) A1:A3`) and what it evaluates to, or
refuses the workbook. references.md "Intersection operands must be references" then lists them as
VERIFIED, and every engine difference is a filed task.

## Context
- references.md "Intersection operands must be references" lists the probed accepted calls (INDEX,
  OFFSET, IF on either side; INDIRECT, CHOOSE, XLOOKUP, SWITCH, IFS, TAKE, DROP, TRIMRANGE, LET,
  `_xlfn.SINGLE`, ANCHORARRAY as right operands) and the probed refused ones (ABS, ROW, IFERROR, IFNA,
  CHOOSEROWS, CHOOSECOLS, FILTER, VSTACK, SORT, UNIQUE, VLOOKUP, HLOOKUP, LOOKUP, MATCH, ROWS, SUM, NA).
  Its "Implemented in" line says: "UNVERIFIED by inference: built-in functions not probed are refused,
  while unknown or future function names, LAMBDA definitions and calls, and `@` operands are accepted".
- Ours: `is_reference_operand` (crates/rustsheet-engine/src/expressions/parser/reference_operators.rs)
  over `Function::may_be_reference` and `picks_reference` (crates/rustsheet-engine/src/functions/util.rs).
- Shapes to settle (with A1:B3 = 1, 2, 3 / 10, 20, 30, as the existing probes):
  1. The accepted right-operand functions as the **left** operand: `=INDIRECT("A2:B2") A1:A3`,
     `CHOOSE`, `XLOOKUP`, `SWITCH`, `IFS`, `TAKE`, `DROP`, `TRIMRANGE`, `LET`.
  2. Unprobed built-ins that return or pass arrays shaped like a range: EXPAND, TOCOL, TOROW, WRAPROWS,
     WRAPCOLS, HSTACK, SORTBY, XMATCH, AREAS, CELL, HYPERLINK, IMAGE, GROUPBY.
  3. Calls Excel cannot know the result kind of: an inline `LAMBDA(x,x)(A2:B2)`, a defined-name LAMBDA
     returning its argument, a LET-bound LAMBDA, an unknown name `FOO(A2:B2)` and a future-function
     spelling (`_xlfn.` plus an unknown name).
- Method: a refusal (`FileCorruptTryRepair`) rejects the whole workbook, so the T-0138/T-0161 method
  applies: scratch `rustsheet-oracle recalc` runs of one formula each to sort accepted from refused, then
  one committed probe case holding every accepted shape. Refused shapes are recorded in references.md
  as the existing ones are (a refused workbook has no values to commit). This needs judgement about
  which refusals are caused by the shape rather than by the builder (check each refused formula loads
  on its own outside the intersection, as the doc does for CHOOSEROWS and IFERROR).

## Acceptance criteria
- [ ] Probe case `probes/t3308-intersection-function-operands` (milestone M2) committed with its
      excel-graph sidecar, holding every accepted shape from groups 1-3 with its value.
- [ ] references.md lists each shape of groups 1-3 as accepted (with case and cell) or refused (with
      the scratch-run date), and drops the "UNVERIFIED by inference" sentence or narrows it to what is
      still unprobed.
- [ ] Every committed cell passes, or the failing cells are known failures recorded by `make
      eval-baseline` (0 regressions). A shape the engine accepts but Excel refuses, or the reverse, is
      listed in a filed engine task (case stem and A1 names in one bullet, decision 0159, or the
      refused formulas for a refusal).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `may_be_reference` or the parser (the follow-up engine task).
- Whitespace in the operator (T-3284, T-3307); the `@` operand cases already probed (T-0149, T-1103).
