---
id: T-1877
title: "Milestone function scope classifies array-member cells by their anchor's formula"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The report's "Milestone function scope (ironcalc-calc)" section (the M2 exit criterion
"ironcalc-calc cells using only M1/M2 functions at 100%") classifies each array-member cell (a
cell inside a legacy array formula's range, other than the anchor) by its anchor's formula, not
leaving it out. Today 1,378 formula-less cells are left out, 46 of them failing, so the criterion
reads "met" without checking them.

## Context
`crates/rustsheet-eval/src/function_scope.rs` `summarize` skips every `CellResult` whose `formula`
is `None` and only counts it under "no formula (left out)" (T-1472). The xlsx snapshot already
knows each member's anchor: `FormulaKind::ArrayMember { anchor }` in
`crates/rustsheet-eval/src/xlsx_snapshot.rs`, set for cells inside an `<f t="array" ref=..>` range.
Neither `CellResult` (runner.rs) nor `function_scope` carries that anchor today.

The 46 failing left-out cells at 2026-10-18 (`target/release/rustsheet-eval snapshot <xlsx>`
gives each anchor):
- 45 are array members whose anchor calls a function after M2: IF_ARRAY C4 (`IF(..,SIN(..),COS(..))`)
  and E4 (`SIN`); SEQUENCE I9:I11, L14:M14 (`_xlfn.SEQUENCE`); POWER `inside_a_lambda!C3`
  (`gelu`, a LAMBDA name, which the catalog does not list, so check how `functions_in` treats it);
  and the LINEST_LOGEST and TREND_GROWTH cells (LINEST, LOGEST, TREND, GROWTH).
- `ironcalc-calc/LOGICAL/IF_ARRAY.xlsx` `Sheet1!C11` is a member of B8's
  `IF(FALSE,A1:A4,B1:C4)`, which calls only IF (M2). So it is in scope, and failing. It fails
  because its precedent C4 holds desktop SIN(4) (T-1876). Precedents are not followed, as for
  the TBILLs cells (T-1472 Outcome).
- POWER `inside_a_lambda!C4` has `formula_kind: normal` and no formula, so it has no anchor and
  stays left out.

Keep one definition: take the anchor from the existing `FormulaKind::ArrayMember`, and the
anchor's formula from the same snapshot. Do not re-parse the xlsx in `function_scope`.

## Acceptance criteria
- [ ] Unit tests in `function_scope.rs`: an array member whose anchor calls only M0–M2 functions
  is scored in scope (passing, then failing). A member whose anchor calls a later function is
  left out of the scored cells. A cell with no formula and no anchor still counts under
  "no formula (left out)".
- [ ] The column header says what is left out now (cells with no formula and no array anchor).
  report.json `function_scope` gains no field that duplicates an existing one.
- [ ] `make eval` on main data: the section lists `ironcalc-calc/LOGICAL/IF_ARRAY.xlsx!Sheet1!C11`
  as failing. "no formula (left out)" counts only cells without an anchor, and the failing count
  among them is 1 (POWER `inside_a_lambda!C4`), unless the run shows otherwise. If it does,
  the PR lists each remaining cell and why it has no anchor.
- [ ] Eval: no baseline change in any suite (reporting only).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Following precedents transitively (the TBILLs and IF_ARRAY C11 cells stay "failing in scope").
- Fixing SIN(4) (T-1876).
- Dynamic-array spill children that the xlsx stores without an array range (POWER C4): file a
  follow-up if they turn out to hide M0–M2 cells.
