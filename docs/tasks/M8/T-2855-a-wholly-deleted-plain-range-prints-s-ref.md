---
id: T-2855
title: "A wholly deleted plain range prints S!#REF! as Excel does, and the spill-extent check follows cells moved by sheet ops"
milestone: M8
lane: engine
status: done
priority: P1
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
The two known failures T-2338's new probe cases left pass.

## Context
- probes/table-edits/plain-range-delete-row-first F!A6: deleting row 4 removes all of `=SUM(S!C4:C4)`;
  Excel prints `=SUM(S!#REF!)` (one `#REF!`, sheet-qualified), and the engine prints `=SUM(#REF!:#REF!)`
  because `stringify_reference` turns each deleted corner into `#REF!` on its own (stringify.rs,
  RangeKind). The probed column delete agrees: probes/table-edits/table-delete-column-qty F!A7
  (`=SUM(S!#REF!)`) is the one cell of that case T-2339 left failing for this reason.
- probes/table-edits/table-delete-row-all-data-below S!C7: the spill-extent check
  (crates/rustsheet-eval/src/spill_extent.rs) scores a filled cell that is neither an input nor recorded
  as a mismatch against empty. A value that a sheet op moved (S!C10 → S!C7) is flagged although
  Excel has it there too (F!A1 `=S!C7` reads 99). The check should follow cells moved by `sheet_ops`,
  or not score sheets the oracle did not record. This is a harness fix: split it into a harness-lane
  task if it cannot ride with the engine change.

## Acceptance criteria
- [x] The three cells (the two above and table-delete-column-qty F!A7) pass and leave eval/baseline/probes.txt (`make eval-baseline`), with no new known failure.
- [x] references.md "Structured references when the table changes" no longer lists the `#REF!:#REF!` gap.
- [x] `make check` green; reviewers approve.

## Out of scope
- Column-delete range shrinking (unprobed).

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. Bit-level criteria still unmet then move to a follow-up
filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a, task ownership as of `main` e5bc60c4): this task owns visibly-off probes cells of ROWS (2), COLUMNS (1) and ROW (1). With T-2339 and T-2848 it lifts ROWS, COLUMNS and ROW.

## Handoff (resumed)
Resolved by the opus run: see the Outcome below (the parser keeps the sheet, `Node::SheetRefErrorKind`).

Done and pushed: the harness half (`moved_input` in crates/rustsheet-eval/src/spill_extent.rs follows a
table's kept last data row; probes/table-edits/table-delete-row-all-data-below S!C7 now passes and left
eval/baseline/probes.txt) and the stringify half (stringify.rs `RangeKind` prints one `S!#REF!` when both
corners are deleted; verified by a debug print that it is reached with `Some("S")`).

Still failing: plain-range-delete-row-first F!A6 and table-delete-column-qty F!A7 still print `=SUM(#REF!)`.
Cause: the displaced formula text is re-parsed, and the lexer (lexer/mod.rs ~915-922) turns `S!#REF!` into the
bare `Node::ErrorKind(REF)`, which prints `#REF!` without the sheet. Fix needs the parser to keep the sheet
qualifier of a sheet-qualified `#REF!` (a new node or a field, touching the Node matches in
bound_variables.rs, rename_defined_name.rs, sheet_edits.rs, static_analysis*, reference_operators.rs), and the
value must stay the `#REF!` error (references.md "A sheet-qualified #REF! is #REF!"). Then run
`make eval-baseline`, `make check`, the `review` skill, and open the PR.

## Outcome
- **Printing.** A wholly deleted plain range now prints one sheet-qualified `S!#REF!`
  (stringify.rs).
- **Parsing.** The lexer keeps the sheet of a sheet-qualified `#REF!`
  (`TokenType::SheetRefError`). The parser holds it as `Node::SheetRefErrorKind`, a node that
  exists only in the tree as written: `strip_spaces` turns it into the bare `#REF!` before
  evaluation and analysis, so `=SUM(S!#REF!)` prints back as typed and still evaluates to `#REF!`.
- **Harness.** The spill-extent check follows a table's kept last data row
  (`moved_input` in spill_extent.rs).
- **Eval delta.** probes known failures go from 2533 to 2531 (plain-range-delete-row-first F!A6
  and table-delete-column-qty F!A7), and the handoff run fixed table-delete-row-all-data-below
  S!C7. No regressions.
