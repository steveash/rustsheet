---
id: T-0027
title: "Show shared formulas as they apply to each cell in eval show/snapshot"
milestone: M0
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
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
