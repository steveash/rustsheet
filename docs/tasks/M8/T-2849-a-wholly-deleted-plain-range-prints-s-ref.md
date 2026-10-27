---
id: T-2849
title: "A wholly deleted plain range prints S!#REF! as Excel does, and the spill-extent check follows cells moved by sheet ops"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The two known failures T-2338's new probe cases left pass.

## Context
- probes/table-edits/plain-range-delete-row-first F!A6: deleting row 4 removes all of `=SUM(S!C4:C4)`;
  Excel prints `=SUM(S!#REF!)` (one `#REF!`, sheet-qualified), and the engine prints `=SUM(#REF!:#REF!)`
  because `stringify_reference` turns each deleted corner into `#REF!` on its own (stringify.rs,
  RangeKind). The probed column delete agrees (table-delete-column-qty: `=SUM(S!#REF!)`).
- probes/table-edits/table-delete-row-all-data-below S!C7: the spill-extent check
  (crates/rustsheet-eval/src/spill_extent.rs) scores a filled cell that is neither an input nor recorded
  as a mismatch against empty. A value that a sheet op moved (S!C10 → S!C7) is flagged although
  Excel has it there too (F!A1 `=S!C7` reads 99). The check should follow cells moved by `sheet_ops`,
  or not score sheets the oracle did not record. This is a harness fix: split it into a harness-lane
  task if it cannot ride with the engine change.

## Acceptance criteria
- [ ] Both cells pass and leave eval/baseline/probes.txt (`make eval-baseline`), with no new known failure.
- [ ] references.md "Structured references when the table changes" no longer lists the `#REF!:#REF!` gap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column-delete range shrinking (unprobed).
