---
id: T-1758
title: "A call of a cell-shaped name is a call of that cell"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`=Ab1(1)` is `#REF!` and FORMULATEXT prints `=AB1(1)`, as in Excel: a callee spelled like an A1 cell reference is a call of that cell, not of an unknown name.

## Context
Found by T-1719: probes/unknown-function-name-spelling A8 (`#REF!`, ours `#NAME?`) and B8 (`=AB1(1)`, ours `=Ab1(1)`); rule in docs/excel-semantics/references.md "Unknown function call names". `parse_named_call` (parser/operands.rs) makes any name that is no function a `NamedFunctionKind`; a cell-shaped one should become a `LambdaCallKind` of the reference (the evaluator already makes a called reference `#REF!`, `call_callee`). Known functions shaped like cells (`LOG10`, `ATAN2`, `DAYS360`) must stay functions: only a name no function lookup knows. Probe first which shapes count (a column past XFD, `$`-anchored, a row past 1048576, R1C1-shaped names like `R1C1(1)` in A1 mode) if the evidence does not cover them.

## Acceptance criteria
- [ ] probes/unknown-function-name-spelling A8 and B8 pass; any new probe cells pass; the rule in docs/excel-semantics/ is updated.
- [ ] `=LOG10(100)` and other cell-shaped function names are unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`parse_named_call` turns a callee spelled like an in-range A1 cell, and no function, into a `LambdaCallKind` of that cell (`#REF!`); `callee_operand` prints a bare cell callee unparenthesized (`AB1(1)`). Probe cells unknown-function-name-spelling A8/B8 now pass; unit test added. Edge shapes (past XFD, `$`, R1C1) remain unprobed and are noted in references.md. Resumed by opus: the rule is A1 grammar only (`is_a1_mode`), so the raw A1 text of a formula that did not parse is no longer read as R1C1 offsets (regression test `unparsed_cell_shaped_call_keeps_its_text_after_a_sheet_rename`). Re-parsing raw text in R1C1 already rewrites R1C1-shaped text on main (`=R[1]C[1]+1` becomes `=B9+1` after a rename), and the stored call form adds `=R[1]C[1](1)` to that class: root cause filed as T-1805 (P2). Edge-shape probes: T-1806. Eval delta: probes +2 cells (A8, B8), 0 regressions.

## Handoff (resumed)
Branch `agent/engine/T-1758-cell-shaped-call` holds a working implementation; A8/B8 pass, `make check` was green at the last commit, security/architecture/quality reviewers APPROVE. `reviewer-excel-fidelity` requested changes three rounds running, so it is handed off (step 5b). What exists: `parse_named_call` turns an ASCII, `$`-free, A1-cell-shaped non-function callee into a `LambdaCallKind` of the reference (operands.rs); `callee_operand` prints a relative sheet-less `ReferenceKind` callee bare (reference_operators.rs); `reference_or_call_operand` lets the R1C1 parser read `R[-7]C[27](1)` back (stored form); tests in test_lambda.rs and test_defined_name_grouped.rs; follow-up probe task T-1806.
Open fidelity finding (MAJOR): the cell-shaped rule in `parse_named_call` also runs in R1C1 mode. A parse-error formula is stored as raw A1 text (stringify.rs ~734), so `=Ab1 (1)` re-parsed after `rename_sheet` becomes `=AC18(1)` (A1 coordinates read as R1C1 offsets), a regression from main. Next: gate the rule on `self.lexer.is_a1_mode()` and add a regression test that puts `=Ab1 (1)` in a cell, renames the sheet and asserts the text is not shifted. Also: record `XFD1(1)` after `insert_columns` (becomes `=#REF!(1)`, `#ERROR!`) in T-1806, and state in references.md that XFD and 1048576 count as cells. Then re-run `make check` and the fidelity reviewer.
Set the Outcome note below to done once finished (the status was reset to todo for the handoff).
