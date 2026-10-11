---
id: T-1346
title: "Probe positive fractional elapsed tokens and near-integer negatives"
milestone: M1
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's output for non-negative fractional amounts and for near-integer negatives under elapsed tokens is established, and the engine matches it.

## Context
T-1338 probed tiny negatives: [s] rounds the magnitude, [h]/[m] truncate it (elapsed_text, formatter/format.rs). Unprobed: non-negative fractions (the engine floors, e.g. TEXT(1.7/86400,"[s]")) and the 1e10 noise snap on negatives (TEXT(-(3-1E-11)/24,"[h]"), TEXT(-(2-1E-9)/1440,"[m]"), -0.49999999999 s). Use the excel-probe skill; docs/excel-semantics/number-formats.md "Elapsed-time tokens".

## Acceptance criteria
- [ ] A probe case with oracle values covers positive fractional [s]/[ss]/[m]/[h] and the near-integer negatives above.
- [ ] The engine matches them or a fix is made here; number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 14 cells in both date systems (probes/elapsed-fractions-1900/-1904). Positive `[s]` rounds and `[h]`/`[m]` truncate, like negatives (the engine floored positive `[s]`), and Excel snaps much wider noise than the 1e-10 guess. `elapsed_text` now takes an `ElapsedUnit` and fits the snaps to the probes. No regressions; 28 new passing cells.
Follow-up: T-1362 probes positive near-integers and large serials.
