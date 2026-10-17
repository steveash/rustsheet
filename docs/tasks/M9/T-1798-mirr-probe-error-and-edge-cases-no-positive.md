---
id: T-1798
title: "MIRR: probe error and edge cases (no positive flows, one flow, rates -1 and below, overflow)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
MIRR's error and edge cases are evidenced by excel-graph and the engine matches them.

## Context
T-1687 fitted MIRR's ordinary case (financial.md "MIRR's evaluation order"); probes-numeric/m9-fin-mirr has no error or edge
cells. `fn_mirr` (functions/financial/mirr.rs) keeps the pre-T-1687 paths, unverified, for: no positive flows (engine: -1), a
single flow, a finance or reinvest rate of -1 (ironcalc-calc MIRR.xlsx E8, E9 are #DIV/0! and pass), rates below -1, a zero or
infinite ratio, and power overflow on long or high-rate flows. Use the excel-probe skill: one probe case, `"expected": {}`, M9 tag.

## Acceptance criteria
- [ ] A probe of the cases above, filled by excel-graph; mismatches fixed in `fn_mirr` and the findings in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
