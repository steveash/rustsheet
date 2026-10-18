---
id: T-1851
title: "Legacy N/T marker: internal function kinds, R1C1 spelling and a node entry for legacy formulas"
milestone: M2
lane: engine
status: done
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
(expressions/parser/operands.rs), next to `_xlfn.SINGLE`, behind one matcher,
`is_legacy_marker_spelling`, shared by both modes. It gives the internal kind in
`LexerMode::R1C1` and plain N/T in `LexerMode::A1`. Only `to_rc_format` prints it. The spec's
"Storage" bullets cover the parse-error path and the defined-name shadowing, which this task
records as a `DEVIATION` unless it finds a spelling Excel refuses as a name. The kernels of
`fn_n` (functions/information/mod.rs) and `fn_t` (functions/text/t_function.rs) take
`legacy: bool`. The internal kinds dispatch to thin wrappers, `fn_n_legacy` and `fn_t_legacy`.
`function_iterator` (test/test_function_iterator.rs) is extended with `Function::INTERNAL`.
`Input::LegacyFormula` (crates/rustsheet/src/edit.rs) moves from `legacy_formula_text` +
`set_user_input` to a new `pub fn set_cell_with_legacy_formula(&mut self, sheet: u32, row: i32,
column: i32, formula: &str) -> Result<(), String>`. It parses, rewrites, and stores through a
`set_cell_with_formula_node` split out of `set_cell_with_formula` (no copy). `legacy_formula_text`
stays.

## Acceptance criteria
- [x] Unit tests for SPEC-0012's T-1851 test plan:
  - R1C1/A1 parsing of the spelling;
  - every print path (RC prints the spelling; display, English, FORMULATEXT, `<f>` and array
    `<f>` print `N(`/`T(`);
  - an A1 call of the spelling is plain N/T, also when a LAMBDA name of that spelling exists
    (the actual behavior, with its `DEVIATION` entry unless a spelling Excel refuses as a name is
    found);
  - a parse-error formula holding the spelling comes back as the same parse error after a
    reload (since T-1805 a parse error is never re-parsed);
  - case and `_xlfn.` variants the matcher rejects never reparse to the internal kind;
  - evaluation of a marked kind from a hand-written RC formula (array 2x1, 1x2, 1x1 → `#VALUE!`;
    range → top-left; scalar unchanged);
  - static result `Scalar`;
  - `Input::LegacyFormula` keeps a marker in the stored tree.
- [x] `function_iterator` checks `Function::INTERNAL`. The catalog and `Function::into_iter` do
  not list the internal kinds.
- [x] fuzz/fuzz_targets/formula_parse.rs also parses the raw input in R1C1 mode and re-parses
  its `to_rc_format` output in R1C1 mode.
- [x] Eval: no change in any suite (probes, ironcalc-calc, poi, fixtures).
- [x] `make check` green; reviewers approve.

## Out of scope
- Placing the marker in the legacy rewrite and its export mirror (T-1713).
- Row/column edits, moves, fill and paste (T-1852).

## Outcome
- **The kinds:** `Function::NLegacy` and `Function::TLegacy` now exist, listed in
  `Function::INTERNAL` (new `functions/kinds.rs`, which also holds `is_n_or_t`, the spelling
  matcher and `returns_reference`).
- **The spelling:**
  - `to_rc_format` stores them as `_RS_LEGACY_N(` / `_RS_LEGACY_T(`. Every other text prints
    `N(` / `T(`.
  - `parse_named_call` reads the spelling back as the internal kind in R1C1 and as plain N/T in
    A1, through one exact, case-sensitive matcher, `Function::from_legacy_spelling`.
- **Evaluation:**
  - `fn_n_legacy` and `fn_t_legacy` call the N/T kernels with `legacy: true`, so an array
    argument is `#VALUE!`. A range is still its top-left cell and a scalar is unchanged.
  - Their static result is `Scalar`, so a marked cell is a plain `CellFormula`.
- **Storing a legacy formula:**
  - `set_cell_with_formula` is split into `parse_cell_formula` and `set_cell_with_formula_node`
    (new `formula_cells.rs`, with `formula_style_index` and `set_formula_node_with_units` taken
    out of `set_user_input` and `update_cell_with_formula`).
  - The new `Model::set_cell_with_legacy_formula` stores the rewritten tree, and
    `Input::LegacyFormula` calls it.
- **Tests and fuzzing:**
  - `function_iterator` checks that the enum is the lookup table plus `INTERNAL`.
  - The fuzz target also parses its input and its `to_rc_format` output in R1C1 mode.
- **Two changes from the spec:**
  - The parse-error path no longer marks anything: T-1805 stores a parse error unparsed.
  - The LAMBDA-name shadowing is recorded as UNVERIFIED (probe pending, T-1912), not as a
    `DEVIATION`, because decision 0052 does not allow a deviation here. SPEC-0012 is amended to
    match.
- **Eval delta:** none (`make eval-check`).
