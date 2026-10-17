---
id: T-0432
title: "Oracle canary: skip zip directory entries under worksheets/, then refill the 182 corpus-bench workbooks they blocked"
milestone: M11
lane: harness
status: todo
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
