---
id: T-1997
title: "Accept a defined name spelled like a cell, as Excel opens such a workbook"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A workbook defining a name spelled like an A1 cell (`NLG1`, `TLG1`) loads, as Excel opens it, and its
formulas read the cell, not the name: probes/legacy-marker-spelling-name-nlg1 and -tlg1 pass.

## Context
Found in T-1936. Excel for the web opens such a workbook; `ISREF(NLG1)` is TRUE, `SUM(NLG1)` 0 and
`NLG1(1)` `#REF!` (references.md "Legacy N and T marker spelling"). Ours refuses the case with
`invalid defined name: Name: Invalid defined name` (`defined_name_slot`, `is_valid_identifier` in
crates/rustsheet-engine/src/defined_names.rs), so both cases are `!EngineError` in the probes
baseline. Check whether the xlsx import takes the same path; the API's refusal of a new cell-like
name may well stay (Excel's Name Manager refuses one), so the fix may belong to the loading path
only. Do not let such a name shadow the cell.

## Acceptance criteria
- [x] Both probe cases pass (baseline shrinks); a unit test loads such a name and reads the cell.
- [x] `make check` green; reviewers approve.

## Out of scope
- The marker spelling itself (T-1936).

## Outcome
New `Model::define_loaded_name` / `Txn::load_defined_name` (the eval harness's loading path) accept a name
spelled like an A1 cell (`is_cell_spelled_name`); `define_name` and `new_defined_name` (Name Manager) still
refuse it. The
lexer reads such a spelling as the cell, so the name never shadows it. xlsx import never validated
names, so it already loaded them. Eval: probes legacy-marker-spelling-name-nlg1 and -tlg1 now pass
(12 cells; two `!EngineError` known failures removed).
