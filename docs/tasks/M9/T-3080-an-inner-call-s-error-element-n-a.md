---
id: T-3080
title: "An inner call's error-element #N/A stays tagged as an array element, so an outer SMALL lets a text k win (SMALL(SMALL({1,#N/A,3},2)/1.005,"a*") is #N/A in Excel, ours #VALUE!)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An error that reaches a statistical function's data argument as a whole value, such as an inner call's result or an
arithmetic result, counts as the data argument's own error. It is then returned before any other argument's error,
as statistics.md "An error element of an array argument is that error" requires ("an error value as the first (data)
argument still comes first"). Only an error element inside an array value still lets a non-coercible k or percent win.

## Context
- Evidence (excel-graph): gen-m9/1b31092a0d79e5f1 Calc!A363 `=SMALL((SMALL({1,#N/A,3},2)/1.005),Data!C8)` with Data!C8
  the text `a*`. Excel gives #N/A and ours gives #VALUE! (ErrorMismatch, a known-failure line in eval/baseline/gen-m9.txt).
  Done tasks T-2430 and T-2636 both list the cell. T-2636's Context leaves it ("not covered here"), and no open task fixes it.
- Cause (traced in the 2026-10-28 gap triage with a scratch case): `array_element_error`
  (functions/statistical/element_errors.rs) marks an element error by its message, "Error in array argument". The inner
  `SMALL({1,#N/A,3},2)` returns that tagged #N/A, and the tag survives `/1.005`. The outer SMALL's
  `lift_over_sorted` (functions/statistical/sorted_data.rs) then sees `is_array_element_error` on its data argument
  and lets k's #VALUE! win. Engine results on that scratch run:
  - #VALUE!, but #N/A by the rule: `SMALL(SMALL({1,#N/A,3},2)/1.005,"a*")`, `SMALL(SMALL({1,#N/A,3},2),C8)`,
    `SMALL(LARGE({1,#N/A,3},2),C8)`, `SMALL(SMALL({1,#N/A,3},2),"x")` (C8 = `a*`).
  - #N/A already, correct: `SMALL(NA(),C8)`, `SMALL(NA()/1,C8)`, `SMALL(IFERROR(1/0,NA()),C8)`, `SMALL(MAX({1,#N/A}),C8)`.
  - Correctly #VALUE!, the element error inside an array losing to a text k (statistics.md, `TRIMMEAN({1,#N/A,3},{"a"})`):
    `SMALL({1,#N/A,3},C8)` and `LARGE({1,#N/A,3},C8)`.
- The same tag is read in prob.rs and by every `array_element_error` caller (LARGE, SMALL, PERCENTILE*, QUARTILE*,
  PERCENTRANK*, TRIMMEAN, MODE*, Z.TEST, the paired functions, F.TEST). Fix it once, where a function's result or an
  operator's result leaves the call (for example, drop the tag when the error becomes a call's value), not per
  function. Search for an existing "without_message" or tag-clearing helper first (`rg without_message`).

## Acceptance criteria
- [ ] gen-m9/1b31092a0d79e5f1 Calc!A363 passes; its line disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure line in any baseline.
- [ ] Unit tests: the four #VALUE! formulas above give #N/A, and the #N/A and #VALUE! controls keep their values.
- [ ] statistics.md's "An error element of an array argument is that error" gets an "Ours:" line naming this task
      (the evidence sentence for A363 is already there).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probing which error wins for an array *value* with an error element that comes from a nested call (for example,
  `SMALL(IF(TRUE,{1,#N/A,3}),"x")`). If the fix changes such a cell, file a probe task instead of guessing.

## Outcome
Done 2026-11-01. The element-error tag (`element_errors::ARRAY_ELEMENT_ERROR`, a message) is dropped in one place: when a
function call's result leaves the call (`Model::evaluate_node_in_context`, the `FunctionKind` arm), so an inner call's
error reaches an outer call, or an operator, as a plain error, and the outer call returns it as its data argument's own.
An element error read from an array literal or range is tagged by the outer call's own reader and still loses to a text
k (`SMALL({1,#N/A,3},"x")` stays `#VALUE!`). `element_errors` is now `pub(crate)`.
- gen-m9/1b31092a0d79e5f1 Calc!A363 passes (its line leaves eval/baseline/gen-m9.txt); every other gated suite
  unchanged, no regressions. SMALL had this as its only visibly-off attributed cell, so it reaches the Phase 1 rate.
- Unit cases in test_fn_array_error_elements.rs: the three nested forms the gap triage listed are `#N/A`; the two
  array-literal forms stay `#VALUE!`.

