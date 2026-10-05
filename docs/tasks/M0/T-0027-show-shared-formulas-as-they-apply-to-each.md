---
id: T-0027
title: "Show shared formulas as they apply to each cell in eval show/snapshot"
milestone: M0
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`rustsheet-eval show` and `rustsheet-eval snapshot` print, for a cell that uses a shared formula, the formula
as it applies to that cell (references shifted), not the anchor cell's text.

## Context
Today a shared-formula member prints the anchor's text: poi/NumberFormatApproxTests.xlsx Tests!A96 shows
`=TEXT(C41, B41)` (formula_kind shared, si 5) although it computes `TEXT(C96, B96)`; poi/45544.xlsx and
poi/47813.xlsx have the same pattern. Readers of `show` output (triagers, reviewers) are misled. Code:
`crates/rustsheet-eval/src/xlsx_snapshot.rs` (formula_kind) and `main.rs` `cmd_show`. Reuse the engine's
existing shared-formula expansion if one exists (`rg -i shared crates/rustsheet-engine crates/rustsheet-xlsx`)
rather than writing a second reference shifter. Found in the T-0019 review.

## Acceptance criteria
- [ ] `show poi/NumberFormatApproxTests.xlsx` prints `=TEXT(C96, B96)` for Tests!A96; a unit test covers a
      shared-formula member with relative and absolute references.
- [ ] The snapshot JSON keeps `formula_kind` (shared, si) so tools can still tell members apart.
- [ ] Eval results unchanged (display only).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to how formulas are evaluated or scored.

## Handoff (resumed)
Sonnet run, no code written; stopped at a design question.
- `xlsx_snapshot.rs` parses raw XML, so the shifter has to work on formula *text*. The engine's shifter
  (`expressions/parser/move_formula.rs`, `pub(crate)`) works on parsed `Node`s and re-stringifies, and
  `Model::get_cell_formula` does the same through `to_localized_string`: both drop the source spacing and
  print `=TEXT(C96,B96)`, whereas the acceptance criterion shows `=TEXT(C96, B96)`.
- No text-preserving shifter exists (`rg` for shift/move/displace found only node-based ones in
  `actions.rs`, `cut_paste.rs`, `move_formula.rs`).
- Options: (a) expose a small public engine helper that shifts a formula's text by (dr, dc) by walking
  `Lexer` tokens and rewriting only reference tokens, then call it from `finish_cell` / the post-pass in
  `parse_sheet` (needs the master's address, which `masters` does not keep yet); (b) relax the criterion to
  accept engine-normalised spacing and use `Model::get_cell_formula` after loading. (a) matches the
  criterion; (b) is smaller but compares less well to the oracle's own text.
- Either way check that `calibrate.rs`, `volatile.rs` and `report.rs` (users of `CellSnapshot.formula`)
  still behave, and keep `formula_kind` unchanged.
