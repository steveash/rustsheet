---
id: T-3548
title: "MMULT with an error value argument is #VALUE! in Excel (engine propagates the error)"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
campaign: semantics
depends_on: []
---

## Goal
MMULT returns `#VALUE!` when an argument is an error value (cell or literal), as Excel does, instead of propagating that error.

## Context
Found by the T-3494 arg-pairs batch (excel-graph oracle). Excel gives `#VALUE!` where the engine propagates the argument's error (`eval_matrix_arg`, matrix.rs, T-2164 covers blanks only).
- gen-m5/842a0aebcb17b83d: A463 `MMULT(Data!H1,#NUM!)` (Excel `#VALUE!`, engine `#NUM!`) and A466 `MMULT(Data!H2,Data!G9)` with G9 = `#DIV/0!` (Excel `#VALUE!`, engine `#DIV/0!`).
- Check MDETERM and MINVERSE in the same probe (coercion.md has their text/blank rules) and record the finding in docs/excel-semantics/arrays.md near the matrix blank rule.

## Visible-parity stage
Decision 0083: a wrong error type is visible to every user of the function; the acceptance criteria below are already the visible-parity stage.

## Acceptance criteria
- [ ] A463 and A466 of gen-m5/842a0aebcb17b83d match Excel; MDETERM/MINVERSE error arguments probed and documented.
- [ ] The cells above leave eval/baseline/gen-m5.txt (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' error handling.
