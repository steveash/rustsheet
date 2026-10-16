---
id: T-1705
title: "Case builder: write pivot table fixtures so GETPIVOTDATA can be probed against a real pivot"
milestone: M6
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The case builder can write a pivot table into a probe workbook so GETPIVOTDATA can be probed against a real pivot.

## Context
T-1628 probed GETPIVOTDATA only without a pivot (builder cannot write pivots); field/item pairs, grand totals and missing items stay unprobed. Needs a design note (tools/ spec->xlsx builder, excel-graph acceptance of pivots).

## Acceptance criteria
- [ ] A probe with a pivot fixture is oracle-filled and covers field/item pairs, grand totals and a missing item; findings in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
