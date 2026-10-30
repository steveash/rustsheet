---
id: T-3113
title: "Add an R1C1 typed-entry mode for spec cells (entered_r1c1 written as a formulasR1C1 PATCH) to the case schema, oracle client and engine adapter"
milestone: M2
lane: harness
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A spec cell can be entered as R1C1-mode typed text, so a probe can ask how Excel parses a formula typed in R1C1 mode.
The case schema, the excel-graph backend and the engine adapter all handle the new mode, and the libreoffice backend
refuses it as it refuses `entered`.

## Context
- T-3102 (whitespace after `!` in an R1C1-mode formula, `=S! R1C2`) cannot be probed today. Its Handoff notes say
  `{"entered": ..}` is written only as a range `values` PATCH (tools/src/rustsheet_tools/spec2xlsx.py
  `entered_cells`, docProps `rustsheet.entered.<n>`; tools/src/rustsheet_tools/oracle/backends.py ~84;
  docs/architecture/oracle.md "Typed input"), and that `rg formulasR1C1` finds nothing in tools/ or crates/.
- Shape to decide (record it in a decision via `make new-decision` if it changes the case schema; check whether
  docs/architecture/eval-harness.md needs a spec section first):
  - a cell `{"entered_r1c1": "=S! R1C2"}`, written by the backend as a range `formulasR1C1` PATCH before
    `calculate`;
  - `CellInput` in crates/rustsheet-eval/src/case.rs (next to `Entered`);
  - the engine adapter (crates/rustsheet-eval/src/engine.rs ~255/~492, `Input::Parse`), which needs to parse the
    text with the lexer's existing `LexerMode::R1C1` relative to the cell, or to refuse the cell if the engine has no
    R1C1 entry path.
- A PATCH that Excel refuses (400 InvalidArgument, as `=S! R1C2` gets in A1 mode) has to be recordable as the
  answer, not as a fill failure. Check how T-2869 dealt with it.

## Acceptance criteria
- [x] One probe cell `{"entered_r1c1": "=R1C2+1"}` round-trips: filled by excel-graph, and the engine matches.
- [x] docs/architecture/oracle.md "Typed input" and the spec2xlsx.py module doc describe the mode.
- [x] T-3102 is unblocked (status todo) in the same PR.
- [x] `make check` green; reviewers approve.

## Out of scope
- The T-3102 probes themselves (whitespace after `!`).

## Outcome
`{"entered_r1c1": "=R1C2+1"}` is a typed entry in R1C1 mode (decision 0179): spec2xlsx records it in
`rustsheet.entered.<n>` with a fourth element `"r1c1"` (A1 entries unchanged, so existing workbooks stay
byte-identical), excel-graph writes a `formulasR1C1` PATCH, excel-desktop `Range.FormulaR1C1`, libreoffice refuses.
The engine adapter converts it to A1 relative to its cell with the new `rustsheet_engine::r1c1_formula_to_a1` (the
lexer's `LexerMode::R1C1`) and types that. A typed entry Excel refuses (400 `InvalidArgument`) is now an answer: the
backend logs `REFUSED` and the cell stays blank; the adapter leaves an entry the engine refuses blank too. T-3102 is
unblocked. Follow-ups: T-3212 (the engine's R1C1 reader lacks bare `R`/`C` offsets, `R[-1]C`) and T-3213 (the
engine stores a typed formula that does not parse as `#ERROR!` where Excel refuses it).
Eval: 2 new probe cases: probes/t3113-entered-r1c1 passes 4/4; probes/t3113-entered-refused passes 1/2 (S!B1, typed
`=1+` in A1 mode, recorded as a known failure for T-3213). No regressions.
