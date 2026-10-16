---
id: T-1729
title: "T over an array lifts element-wise in a dynamic formula"
milestone: M2
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A dynamic `=T(IF(1,{"a";"b"}))` spills "a", "b", as in Excel, instead of one "".

## Context
probes/legacy-array-result S!G1:G2 (excel-graph, 2026-10-16, T-1713). T lifts over an array value
element-wise (expected: text kept, anything else ""; unprobed), the way N does since T-1616 (information.md "N over an
array lifts element-wise", the N arm in functions/information/mod.rs), while T of a multi-cell
range is still one "" with no spill (text.md "Text function argument modes", references.md "T of a
multi-cell range is not intersected", probes/text-argument-modes F9; keep that). Only this
one T-over-array cell is probed: add an error element and a number element (`T({"a",1,#N/A})`)
with excel-probe before generalizing. information.md "T over an array lifts element-wise".

## Acceptance criteria
- [x] probes/legacy-array-result S!G1:G2 leave eval/baseline/probes.txt; probes/text-argument-modes F9 still passes; no regressions
  (except the two legacy spill rows of decision 0069).
- [x] information.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The legacy `=T(IF(...))` `#VALUE!` (T-1713).

## Handoff (resumed)
Tried (committed on this branch): `fn_t` (functions/text/common.rs) lifts `CalcResult::Array` element-wise
(text and error elements kept, others ""), `static_analysis.rs` result-shape arm `Function::T` changed from
`not_implemented(args)` to `scalar_arguments(args)` (as N), unit test `t_lifts_over_an_array` in
test_cell_info_n_sheets.rs, information.md updated. Unit tests pass; the change fixes probes/legacy-array-result
S!G1:G2 (dynamic).

Failed: `rustsheet-eval check` reports two REGRESSIONS, probes/legacy-array-result S!G11 and
probes/legacy-n-t-reference-args S!S11: those are legacy (non-dynamic) `=T(IF(1,{...}))` cells whose spill rows must stay
empty (Excel gives `#VALUE!`, T-1713), and ours now spills into them, exactly as N does in the baselined
legacy-n-t-reference-args G10:G11. The engine has no legacy-formula marker (spec T-1727), so fn_t cannot tell a
dynamic from a legacy cell. `make eval-baseline` refuses the regressions; I did not touch eval/baseline.
Next: either land the T-1727 legacy marker first (T-1713) so legacy T/N cells give `#VALUE!`, or decide with a
decision record how to baseline G11/S11 (protected path, hence opus). The number/error element rule is still INFERRED:
probe `T({"a",1,#N/A})` with excel-probe and update information.md.
Also: CI `hygiene` fails because functions/text/common.rs is now 1008 lines (limit 1000, check_file_size.py). Put the array lift
in a helper in another text module (or move fn_t out) rather than growing common.rs.

## Outcome
- `fn_t` moved to its own module (functions/text/t_function.rs, keeping common.rs under the
  1000-line limit) and lifts over an array with N's `map_elements`: text and error elements kept,
  numbers and booleans "". The static analysis gives T scalar arguments, as N.
- New probe probes/t-array-lift (excel-graph) settles the element rule (was INFERRED) and shows that
  a dynamic `T(range)` reads the top-left cell (S!E7 "x"; ours ""): recorded in references.md and
  added to T-0985, which owns it.
- Decision 0069: the legacy spill rows probes/legacy-array-result S!G11 and
  probes/legacy-n-t-reference-args S!S11 passed only because T never spilled; they are baselined
  under T-1713 (G11 added to its acceptance list), as N's G11:I11 already are.
- Eval: probes/legacy-array-result S!G1:G2 fixed; t-array-lift 21 of 22 cells pass; 2 decision-0069
  lines added.
