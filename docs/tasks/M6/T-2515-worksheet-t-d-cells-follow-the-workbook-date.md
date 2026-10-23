---
id: T-2515
title: "Worksheet t=d cells follow the workbook date system (1904 workbooks)"
milestone: M6
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
In a `date1904` workbook, a worksheet cell `t="d"` holds the serial of its date in the 1904 system.

## Context
`crates/rustsheet-xlsx/src/import/worksheets.rs` reads `t="d"` with `iso8601_to_serial`, a 1900-system serial, and ignores the workbook's date system (found in T-2393, which fixed the same for pivot cache `<d>` items with `DateSystem::serial_from_imported_1900_serial`; reuse it). Needs Excel evidence for a date before the 1904 epoch (a probe via the `excel-probe` skill), which is why this is opus.

## Acceptance criteria
- [x] A unit test: a 1904 workbook's `t="d"` cell `2024-01-01T00:00:00` is 43830.0, a 1900 workbook's is 45292.0.
- [x] The pre-epoch case is probed and recorded in docs/excel-semantics/.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed five `date1904` fixtures on Excel for the web (fixtures/t-d-1904*): a `t="d"` cell holds its
date's 1904 serial (`2024-01-01T00:00:00` = 43830, `1904-01-01` = 0), a time of day alone stays a day
fraction (0.5), and a date before 1904-01-01 (`1903-12-31`, `1900-01-01`) is `#VALUE!`, as a pre-1900
date is in a 1900 workbook (docs/excel-semantics/xlsx-storage.md). New `iso8601_to_date_system_serial`
in `import/iso_date.rs` is the one conversion: worksheet cells (values and cached formula values),
pivot cache `<d>` items and the eval snapshot reader use it. Unit test
`t_d_cells_take_their_serial_in_the_workbook_date_system`. Eval: five new fixture cases, all passing.
