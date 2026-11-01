---
id: T-3548
title: "MMULT with a blank first argument is #VALUE! even when the second argument is an error"
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
`MMULT(<blank>, <error>)` returns `#VALUE!` as Excel does: the first argument's blank check runs before the second argument's error is propagated.

## Context
Found by the T-3494 arg-pairs batch (excel-graph oracle), gen-m5/842a0aebcb17b83d: A463 `MMULT(Data!H1,#NUM!)` (Excel `#VALUE!`, engine `#NUM!`) and A466 `MMULT(Data!H2,Data!G9)` with G9 = `#DIV/0!` (Excel `#VALUE!`, engine `#DIV/0!`); H1 and H2 are blank.
- Rule: docs/excel-semantics/arrays.md "MMULT: a blank first argument is #VALUE! even when the second is an error". An error in the second argument otherwise passes through (A19, A22, A67, A475), an error first argument beats a blank second (A418), and a text first argument does not beat the second's error (A208); about 40 such cells pass today and must stay passing.
- Code: `Model::eval_matrix_arg` (matrix.rs, T-2164): check argument 1 for blank before argument 2's error is returned.

## Visible-parity stage
Decision 0083: a wrong error type is visible to every user of the function; the acceptance criteria below are already the visible-parity stage.

## Acceptance criteria
- [ ] A463 and A466 of gen-m5/842a0aebcb17b83d match Excel, and every other MMULT cell of that case that passes today still passes.
- [ ] MDETERM/MINVERSE with a blank argument beside an error probed and documented if different.
- [ ] The cells above leave eval/baseline/gen-m5.txt (`make eval-baseline`, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' error handling.
