---
id: T-2855
title: "A wholly deleted plain range prints S!#REF! as Excel does, and the spill-extent check follows cells moved by sheet ops"
milestone: M8
lane: engine
status: in-progress
priority: P1
size: S
tier: sonnet
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
- [ ] The three cells (the two above and table-delete-column-qty F!A7) pass and leave eval/baseline/probes.txt (`make eval-baseline`), with no new known failure.
- [ ] references.md "Structured references when the table changes" no longer lists the `#REF!:#REF!` gap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column-delete range shrinking (unprobed).

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. Bit-level criteria still unmet then move to a follow-up
filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a, task ownership as of `main` e5bc60c4): this task owns visibly-off probes cells of ROWS (2), COLUMNS (1) and ROW (1). With T-2339 and T-2848 it lifts ROWS, COLUMNS and ROW.
