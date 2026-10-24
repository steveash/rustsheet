---
id: T-2409
title: "TOROW and UNIQUE read a lone blank cell as a blank, not 0 (gen-m7)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
TOROW and UNIQUE given a lone blank cell (a one-cell reference, not a range) treat it as the blank it is, as Excel does: TOROW's ignore
1 or 3 drops it (so `TOROW(Z1,3)` is `#CALC!`, nothing left), and UNIQUE's result element is the empty value (`UNIQUE(Z1)&"x"` is "x").

## Context
Gap triage of the T-2254 gen-m7 sweep (excel-graph, 2026-10-22); 2 cells.
- `TOROW(Data!Z1,3)` (Z1 blank) is `#CALC!` (9bb61f6ae34c7e20 A3029); ours 0. `TOROW(Z1:Z1,3)` and `TOROW(Z1:Z2,3)` are already `#CALC!`
  in ours, so only the lone cell reference is read as the value 0 before the blank test.
- `TEXTAFTER(UNIQUE(Data!Z1,Data!A$2)&IF(TRUE,Data!E5,"a"),MID(Data!$B12,6,5))` is "1000000000000000" (f7e94a817e5385ac A2942): MID of the
  blank B12 is "", so TEXTAFTER returns the whole text (text.md "TEXTBEFORE and TEXTAFTER with the empty delimiter"), and UNIQUE of the
  blank Z1 (by_col TRUE) joins as "". Ours "01000000000000000": `UNIQUE(Z1)&"x"` is "0x", while `UNIQUE(Z1:Z2)&"x"` is "x" (checked
  2026-10-22). This agrees with the empty element UNIQUE already gives in its lifted form (arrays.md "UNIQUE's and SORTBY's lifted
  elements": `&"|"` is "|").
The same reading (a lone blank reference as 0) shows in ours for TOCOL (`TOCOL(Z1,1)` 0), SORT (`SORT(Z1)&"x"` "0x") and TAKE (`TAKE(Z1,1)&"x"`
"0x"); those are unprobed, so change them only if the shared fix (a one-cell reference read as a 1x1 range) covers them, and record
them in the quirk entry as following by analogy, UNVERIFIED. Code: `fn_torow`/`lift_flatten` (functions/lookup_and_reference/tocol_torow.rs),
`fn_unique` (functions/spill_functions.rs). Quirk entry: docs/excel-semantics/arrays.md "TOROW and UNIQUE of a lone blank cell" (added by
this triage).

Owned gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22): 2 known-failure lines in eval/baseline/gen-m7.txt, 2 formulas.
- gen-m7/9bb61f6ae34c7e20.case.json A3029 `=TOROW(Data!Z1,3)`: A3029 Excel #CALC!, ours 0.0; cells (1): A3029
- gen-m7/f7e94a817e5385ac.case.json A2942 `=TEXTAFTER((UNIQUE(Data!Z1,Data!A$2)&IF(TRUE,Data!E5,"a")),MID(Data!$B12,6,5))`: A2942 Excel "1000000000000000", ours "01000000000000000"; cells (1): A2942

## Acceptance criteria
- [ ] Visible parity: these 2 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new
      known-failure lines in any baseline.
- [ ] Unit tests: `TOROW(Z1,3)` is `#CALC!`, `UNIQUE(Z1)&"x"` is "x", and a non-blank lone cell (`TOROW(A1,3)` with A1 = 5) is 5.
- [ ] The arrays.md entry's "Ours:" line names this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Blank elements of computed arrays handed to LAMBDAs (T-2413).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
TOROW, TOCOL and UNIQUE read a lone blank cell reference as blank (`Model::eval_to_array_keep_blank`), not 0. Unit test
`torow_lone_blank_cell_is_ignored`. Eval: 5 known-failure lines removed (2 gen-m7 owned, 3 gen-m9), none added. SORT and TAKE
untouched (unprobed).
