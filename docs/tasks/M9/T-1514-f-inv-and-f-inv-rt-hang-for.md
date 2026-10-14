---
id: T-1514
title: "F.INV and F.INV.RT terminate for a huge numerator df (F.INV(0.5,1E10,4))"
milestone: M9
lane: functions-stat
status: todo
priority: P0
size: M
tier: sonnet
depends_on: []
---

## Goal
`F.INV` and `F.INV.RT` return Excel's value for a very large degrees-of-freedom argument instead of running past the harness timeout.

## Context
T-1049 found that `=F.INV(0.5,10000000000,4)` and `=F.INV.RT(0.5,10000000000,4)` did not finish in 60 s (probes-numeric timeout); Excel returns 1.1916487540808696 and 1.191648723307067. A formula in an untrusted workbook must not hang (CLAUDE.md directive 5). Evidence: docs/excel-semantics/statistics.md section "Modern distribution functions ... (T-1049)". Re-add the cell `=F.INV(0.5,1E10,4)` / `=F.INV.RT(0.5,1E10,4)` as a unit test or a small probes-numeric case with its oracle value.

## Acceptance criteria
- [ ] Both formulas return within 1 s; match Excel's value to the last digits the kernel allows, or a DEVIATION entry under decision 0014 if the value cannot be reached within the limit.
- [ ] A regression test covers df1 = 1E10 and df2 = 1E10 for both functions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other F.* last-digit misses.
