---
id: T-2749
title: "Probe DB's boolean month (typed and from a cell, with a valid period) and PV/NPER's inferred domain lines from T-2620"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the T-2620 rules that are inferred rather than probed, so financial.md can mark them VERIFIED (or the
engine follows what Excel does).

## Context
T-2620 made DB read a boolean `month` as a number. The only evidence is `DB(15779,385,37,<FALSE>,<FALSE>)` = `#NUM!`
(probes/t2475-stat-subexpressions P!A165), where the period is FALSE as well. It also added two lines inferred from
Excel's formulas (financial.md "Annuity and DB argument domains at their limits"). Probe with the excel-probe skill:
- `DB(10000,1000,6,1,TRUE)`, `DB(10000,1000,6,1,FALSE)`, and the same with TRUE/FALSE cells (valid period 1);
- `PV(-1.5,400,-100)` (P underflows at a rate in (-2,-1)), `PV(-1.5,401,-100)`;
- `NPER(1E-300,0,100,200)` (LN of a negative ratio before the ln(1+rate) division).

## Acceptance criteria
- [ ] A probes case with these cells, filled from excel-graph; financial.md's Status line updated with the cells.
- [ ] Any cell that disagrees is fixed or has a task filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD and PMT cells (T-2747, T-2748).
