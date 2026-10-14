---
id: T-1565
title: "CHOOSE with an array index and a range value argument broadcasts the range in a dynamic formula"
milestone: M2
lane: functions-lookup
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`CHOOSE({1,1},A1:A4)` (array index, range value argument) in a dynamic formula is read by Excel as the range repeated per index element (a 4x2 broadcast is a hypothesis: probe the spilled shape first); the engine intersects the range per element and gives `#VALUE!` off-row.

## Context
probes/stat-a-computed-arrays B13, B14, B15 (excel-graph, T-1559): `AVERAGEA`, `MAXA`, `MINA` over it are 2, 4, 2 (engine `#VALUE!`). Code: `choose_element` in functions/lookup_and_reference/mod.rs reduces a chosen range by implicit intersection. Distinct from T-0294 (legacy formulas under SUMPRODUCT); check the legacy case does not regress.

## Acceptance criteria
- [ ] Probe the spilled shape of `CHOOSE({1,1},A1:A4)` first; B13-B15 of probes/stat-a-computed-arrays pass and leave the baseline; the rule is in lookup.md, linking back to statistics.md "AVERAGEA, MAXA and MINA over an array".
- [ ] `make check` green; reviewers approve.

## Out of scope
-
