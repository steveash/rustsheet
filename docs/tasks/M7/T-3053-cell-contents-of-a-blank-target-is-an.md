---
id: T-3053
title: "CELL('contents') of a blank target is an empty value that array arguments reject, through LET/IF/IFERROR/CHOOSE (T-3030 evidence)"
milestone: M7
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
`CELL("contents", ref)` of a blank target behaves as Excel's empty value wherever it flows: array-taking functions
reject it with `#VALUE!` even through LET, IF, IFERROR, CHOOSE, parentheses, `@` and an info type read from a cell,
while a blank reference read through the same wrappers stays a blank that stacks.

## Context
- Evidence (excel-graph, T-3030): probes/t3030-cell-contents-blank-shapers and probes/t2466-cell-contents-stack. The
  blank-target result has ISBLANK TRUE, TYPE 1, COUNTA 1, `+0` 0 and `&""` "" (A1, A11, A41, A51, A231). VSTACK, HSTACK,
  TOCOL, TOROW, TAKE, WRAPROWS, CHOOSECOLS, FILTER, MAP and ROWS reject it, as does VSTACK around an INDEX of it (A121; whether INDEX itself rejects it is unprobed) with `#VALUE!` (A21, A31, A121,
  A171, A191, A211, A311, A351, A381), and so does VSTACK through IF, LET, CHOOSE, IFERROR, `@` and parentheses
  (A131, A141, A161, A331; t2466 A111, A121, A151). SORT, UNIQUE and EXPAND accept it as 0 (A181, A201, A371).
- docs/excel-semantics/information.md, "CELL("contents") in VSTACK" entry.
- T-3030 catches only a literal `CELL("contents", ref)` stack argument (`blank_cell_contents` in
  crates/rustsheet-engine/src/functions/lookup_and_reference/hstack_vstack.rs). Ours reads a blank reference through
  IF/LET/CHOOSE/IFERROR/parentheses as a bare `CalcResult::EmptyCell`, the same value CELL returns. So a value-based
  rule would also break `VSTACK(IF(TRUE,Z1),1)`, where Excel stacks a blank reference. This needs a distinct
  non-reference empty value (a design question: a new `CalcResult` variant, or keeping wrapper results as references).
  Probe a bare `INDEX(CELL("contents",Data!C11),1)`, `VSTACK(IF(TRUE,Z1),1)`, `VSTACK(LET(x,Z1,x),1)` and `VSTACK(CHOOSE(1,Z1),1)` first, since they are unprobed.

## Acceptance criteria
- [ ] probes/t2466-cell-contents-stack A111, A121 and A151 pass, as do the rejecting cells of
      probes/t3030-cell-contents-blank-shapers listed above, its A231 (`COUNTA(..)` is 1 in Excel, 0 in ours) and
      probes/t3030-cell-contents-nested-sheet A231 (TOCOL of a blank target). No blank-reference stack cell regresses.
- [ ] information.md "Ours" updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- CELL's other info types.

## Progress (2026-10-28, probe done; implementation not started)
The unprobed shapes are now probed: probes/t3053-blank-reference-wrappers (excel-graph, 2026-10-28; information.md, the
paragraph on a blank reference read through a wrapper). A blank reference through IF, LET, CHOOSE, parentheses, `@` and
INDEX stacks as 0 (ours matches). IFERROR and IFNA of a blank reference return the rejected empty value
(`VSTACK(IFERROR(Data!C11,5),1)` is `#VALUE!`, A41, A141), the CELL result stays the empty value through LET and IF (A161,
A171), and `INDEX(CELL("contents",Data!C11),1)` is `#VALUE!` (A1). So the empty value comes from value-returning calls
(CELL, IFERROR, IFNA), and the reference-returning wrappers pass a blank reference on unchanged. The 8 new known failures
(A41, A42, A141, A142, A161, A162, A171, A172) are in eval/baseline/probes.txt and belong to this task too.
