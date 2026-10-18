---
id: T-1851
title: "Legacy N/T marker: internal function kinds, R1C1 spelling and a node entry for legacy formulas"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1727]
spec: docs/specs/SPEC-0012-legacy-n-t-marker.md
---

## Goal
The engine can hold, store and evaluate a legacy N or T call. These are the internal kinds
`Function::NLegacy` and `Function::TLegacy`: an array argument is `#VALUE!`. They print as N/T
everywhere except the R1C1 storage text. `Input::LegacyFormula` stores its rewritten tree directly,
so a marker would survive. Nothing places a marker yet (T-1713 does), so no eval result changes.

## Context
SPEC-0012, "The marker", "Storage", "Entry points" and "Evaluation". The sites to change are the
10 lines of `rg "Function::(N|T)\b"` (functions/mod.rs names and dispatch,
static_analysis.rs signature and static result, static_analysis/legacy.rs `leaves_call_whole`
and `reads_reference_whole`). The reserved call spelling goes in `parse_named_call`
(expressions/parser/operands.rs), next to `_xlfn.SINGLE`: internal kind in `LexerMode::R1C1`,
plain N/T in `LexerMode::A1`. It is printed only by `to_rc_format`. `fn_n`
(functions/information/mod.rs) and `fn_t` (functions/text/t_function.rs) take `legacy: bool`.
`function_iterator` (test/test_function_iterator.rs) is extended with `Function::INTERNAL`.
`Input::LegacyFormula` (crates/rustsheet/src/edit.rs) moves from `legacy_formula_text` +
`set_user_input` to a new `Model::set_cell_with_legacy_formula`, which parses, rewrites and stores
the node through `shared_formula_index`.

## Acceptance criteria
- [ ] Unit tests for SPEC-0012's T-1851 test plan:
  - R1C1/A1 parsing of the spelling;
  - every print path (RC prints the spelling; display, English, FORMULATEXT, `<f>` and array
    `<f>` print `N(`/`T(`);
  - a LAMBDA name spelled like the marker survives a reload;
  - evaluation of a marked kind from a hand-written RC formula (array 2x1, 1x2, 1x1 → `#VALUE!`;
    range → top-left; scalar unchanged);
  - static result `Scalar`;
  - `Input::LegacyFormula` keeps a marker in the stored tree.
- [ ] `function_iterator` checks `Function::INTERNAL`. The catalog and `Function::into_iter` do
  not list the internal kinds.
- [ ] fuzz/fuzz_targets/formula_parse.rs also re-parses its `to_rc_format` output in R1C1 mode.
- [ ] Eval: no change in any suite (probes, ironcalc-calc, poi, fixtures).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Placing the marker in the legacy rewrite and its export mirror (T-1713).
- Row/column edits, moves, fill and paste (T-1852).
