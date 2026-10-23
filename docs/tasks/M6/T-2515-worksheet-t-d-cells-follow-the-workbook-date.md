---
id: T-2515
title: "Worksheet t=d cells follow the workbook date system (1904 workbooks)"
milestone: M6
lane: engine
status: in-progress
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
- [ ] A unit test: a 1904 workbook's `t="d"` cell `2024-01-01T00:00:00` is 43830.0, a 1900 workbook's is 45292.0.
- [ ] The pre-epoch case is probed and recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
