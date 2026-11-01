---
id: T-3628
title: "Split sheet_data.rs (at the 1,000-line limit): move Row and its search into their own module"
milestone: M11
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`sheet_data.rs` has room to change again: the `Row` type, its search (`position`, the dense-offset guess
of T-3608) and its insert move to their own module, with no behaviour change.

## Context
After T-3606, T-3608 and T-3624, `crates/rustsheet-engine/src/sheet_data.rs` is at exactly the 1,000 lines
`scripts/check_file_size.py` allows. The architecture review of T-3624 asked for this split as a follow-up.
The quality review of T-3624 suggested one helper for the column bound that `set_cell`, `set_row` and
`check_layout` each state. It fits here, once there is room for it.

## Acceptance criteria
- [x] `Row` and its methods live in their own module (`sheet_data/row.rs`); `sheet_data.rs` is well under the
      limit.
- [x] One definition of "a column on the sheet" (`row::on_sheet_column`) serves `set_cell`, `set_row` and
      `check_layout`.
- [x] Behaviour unchanged: existing tests pass unmodified.
- [x] `make check` green; reviewers approve.

## Out of scope
- The row layout or its serialised form.

## Outcome
- **New layout:** `sheet_data.rs` goes from 1,000 lines to 625.
  - `sheet_data/row.rs` (61 lines): `Row`, `position`, `insert`, `LINEAR_SEARCH_LIMIT`, and `on_sheet_column`,
    the one column bound. `Row`, its `cells` field and its methods are `pub(super)`.
  - `sheet_data/test.rs` (333 lines): the file's own test module, moved as is. It now imports `LAST_COLUMN`,
    which `sheet_data.rs` no longer needs.
- **Review follow-ups:** `set_cell` now uses `if !on_sheet_column(column) { return None; }`, replacing
  `then_some(())?` as the T-3624 quality review asked.
- **Tests:** the 20 `sheet_data` tests pass unchanged.
