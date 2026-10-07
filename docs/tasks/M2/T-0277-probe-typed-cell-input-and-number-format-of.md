---
id: T-0277
title: "Probe typed cell input and number format of (1) and (1,000)"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what Excel stores, and with which number format, when `(1)` and `(1,000)` are typed into a cell.

## Context
T-0259 made `parse_formatted_number` read `(n)` as -n; typed cell input (`set_user_input`, model.rs) shares it, so typing `(1)` now stores -1 with no format and `(1,000)` stores -1000 with `#,##0`. Neither the value nor the format of typed input is probed (Excel may use an accounting format such as `0_);(0)`). Also probe `( 1 )`, `(-1)`, `($1)` and `(1%)` as text coercion and as typed input. Record in docs/excel-semantics/coercion.md.

## Acceptance criteria
- [x] (text coercion half; typed input split to T-0280) A probe case holds oracle values (and formats, via TEXT or CELL) for each form; coercion.md records them; an engine task is filed if the engine differs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/paren-negative-variants (excel-graph): `( 1 )`, `($1)`, `(1%)`, `(1.5)` coerce to negatives; `(-1)`, `((1))`, `(1)x` are #VALUE!. Recorded in coercion.md. The engine differs for padding, currency and percent: filed T-0279. Typed cell input cannot be probed through case files (strings are text cells): filed T-0280 (opus). Eval: probes gains 1 case with 9 new-coverage known failures (the engine differences above), 0 regressions.
