---
id: T-0432
title: "Oracle canary: skip zip directory entries under worksheets/, then refill the 182 corpus-bench workbooks they blocked"
milestone: M11
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The oracle can embed its canary in workbooks whose zip lists directory entries (`xl/worksheets/`), so the 182 corpus-bench workbooks refused for that get Excel values.

## Context
T-0427's fill refused 182 corpus-bench workbooks with "cannot embed the canary: unreadable xl/worksheets/
(no element found: line 1, column 0)" (list: eval/suites/corpus-bench/PENDING.md). Their archives carry explicit
zero-byte directory entries such as `xl/worksheets/` (e.g. corpus-bench/10747). `formula_parts`
(tools/src/rustsheet_tools/oracle/package.py) adds every name under the workbook's `worksheets/` prefix,
including that directory entry, and `parse_part` fails on its empty content. A directory entry is not an OPC
part and holds no formulas. Keep the scan fail-closed: skip only names ending in `/` with no data.

## Acceptance criteria
- [ ] A tools unit test: a package with a zero-byte `xl/worksheets/` entry embeds the canary; a `/`-named entry with data is still scanned (or refused).
- [ ] `rustsheet-oracle fill -b excel-graph --suite corpus-bench` refilled; PENDING.md regenerated; corpus-bench baseline updated (`make eval-baseline`), rates in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the new failures (gap-triager tasks).

## Outcome
`formula_parts` now skips zero-byte `/`-named zip entries under `worksheets/` (a `/`-named entry with data is still scanned and refused if unreadable); two tools unit tests added. `rustsheet-oracle fill -b excel-graph --suite corpus-bench` wrote 182 new sidecars (the previously blocked workbooks); PENDING.md drops that section. Corpus-bench: 174 new cases, known failures 17 -> 26 (9 new-coverage failures, 0 regressions); engine fixes are for gap-triager tasks.

Reconciliation: of the 182 sidecars, 7 have empty `sheets` (18137, 33094, 342-46, 41589, 44296, 58613, 59884: formula cells dropped by existing fill policy) and 17049 was already a baselined `!Timeout`, so 182 - 7 - 1 = 174 new cases. The 9 new-coverage failures are left to gap-triager.
