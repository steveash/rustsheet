---
id: T-1488
title: "ASC and DBCS lift over a range (spill per element, #SPILL! when blocked); follow-up of T-1248"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
DBCS and ASC lift over a range argument like Excel: one result per element, spilled, and `#SPILL!` when the spill range is blocked.

## Context
Probes/text-modes-dbcs-lift (excel-graph, T-1248): `DBCS(A3:A5)` over 5, a half-width katakana text and a katakana text spills "5", "ｱｲｳ", "カタカナ" (ours `#VALUE!`); `DBCS(A1:A2)` and `ASC(A1:A2)` spill both elements (ours apply implicit intersection, so the same-row element is returned and the second cell is empty: G2, M2, O2). probes/text-modes-dbcs-phonetic-b C8: `DBCS(A11:A12)` with the spill range blocked is `#SPILL!` (ours `#VALUE!`, no row intersection). Code: fn_dbcs and fn_asc in functions/text/char_code.rs; find how CODE and VALUETOTEXT lift (T-1212) and reuse it.

## Acceptance criteria
- [ ] The listed probe cells pass; `make eval-baseline` records the fixes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ASC over INDEX(..) results (T-1411).
