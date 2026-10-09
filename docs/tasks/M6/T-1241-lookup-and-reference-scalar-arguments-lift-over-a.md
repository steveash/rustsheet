---
id: T-1241
title: "Lookup and reference scalar arguments lift over a range or an array (ADDRESS, INDIRECT, XLOOKUP, XMATCH, LOOKUP, OFFSET and more)"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
The probed scalar arguments of ADDRESS, HYPERLINK, INDIRECT, LOOKUP, XLOOKUP, XMATCH, OFFSET, TRIMRANGE and FORMULATEXT lift over a range and an array, CHOOSECOLS/CHOOSEROWS read a range or array of indexes whole as a list, and vector/array arguments take array literals whole, as Excel does.

## Context
T-1238's probe probes/lookup-argument-modes (excel-graph, 2026-10-09) shows the probed scalar arguments lift (CHOOSE already
does), CHOOSECOLS/CHOOSEROWS take a range or array of indexes whole as a list in any orientation (F7-F9: 3x2, 3x2,
2x2; do not lift them), and array literals are taken whole; see docs/excel-semantics/lookup.md "Lookup and reference function argument modes".
Failing rows (F/G/H cells): 1-4 ADDRESS, 7-10 CHOOSECOLS/CHOOSEROWS, 11-13 HYPERLINK, 14-15 INDIRECT, 16-17 LOOKUP, 20-21
OFFSET, 22-23 and 25 XLOOKUP, 26-27 and 29 XMATCH, 34-35 TRIMRANGE, 38 FORMULATEXT. Reuse the existing lift helpers (as the
T-1212/T-1218 lifts did). OFFSET and INDIRECT lifted over arrays return arrays of references; check how the engine
represents that before choosing a design (references.md). Split per function family if it outgrows size M.

## Acceptance criteria
- [ ] The listed probes/lookup-argument-modes cells pass (except those needing T-1242); baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
