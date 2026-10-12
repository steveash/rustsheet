---
id: T-1362
title: "Elapsed tokens: probe positive near-integers and large serials ([s] overflows i32)"
milestone: M1
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Elapsed tokens give Excel's text for positive near-integers and for serials beyond 2^31 seconds.

## Context
`elapsed_text` (formatter/format.rs) casts the count with `as i32`, so `TEXT(30000,"[s]")` saturates at 2147483647 (Excel: 2592000000 by arithmetic; probe it). The 1e-12 / millisecond snaps are also applied to positives by symmetry; every near-integer probe cell (probes/elapsed-fractions-1900/-1904 A9-A13) is negative. Probe `TEXT((3-1E-7)/24,"[h]")`, `TEXT((2-1E-9)/1440,"[m]")`, `TEXT((5-1E-6)/86400,"[s]")`, `TEXT(0.49999999999/86400,"[s]")` and large `[s]`/`[m]` serials; use the excel-probe skill. Reviewer notes on T-1346.

## Acceptance criteria
- [ ] A probe case with oracle values covers the cells above in both date systems.
- [ ] The count uses a wide enough integer type (no saturation), the engine matches the probes, and number-formats.md is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed positive near-integers and large serials (probes/elapsed-positive-large-1900/-1904, 17 cells each, excel-graph): the snaps match Excel for positives; the count overflowed i32. `elapsed_text` now counts in i64. Both cases pass (34 cells newly checked, 0 known failures); number-formats.md updated.
