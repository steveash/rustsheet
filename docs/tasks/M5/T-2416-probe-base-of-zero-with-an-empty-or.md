---
id: T-2416
title: "Probe BASE of zero with an empty or blank min_length (BASE(0,2,), BASE(0,2,Z99))"
milestone: M5
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's BASE result for a zero number with a written-empty or blank-cell `min_length` is known from oracle cells and recorded in docs/excel-semantics/numbers.md.

## Context
Found by the T-2374 Excel-fidelity review: the engine reads an empty `min_length` as 0, so `BASE(0,2,)` and `BASE(0,2,Z99)` now give empty text (they gave "0" before T-2374), but no oracle cell covers it. Probe `BASE(0,2,)`, `BASE(0,2,Z99)`, `BASE(0,2,"")` and `BASE({0,5},2,{0,0})`; fix BASE (mathematical.rs `fn_base`) if Excel differs.

## Acceptance criteria
- [ ] A probes case with those cells, oracle-filled (excel-graph), in eval/suites/probes.
- [ ] numbers.md "BASE of zero with min_length 0 is empty" states the result (replace its Unprobed line).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing BASE unless the probe contradicts the engine (then a fix task).

## Outcome
Probe probes/base-zero-empty-min-length filled by excel-graph. Excel: `BASE(0,2,)` is "0" (written-empty = omitted), `BASE(0,2,Z99)` "", `BASE(0,2,"")` `#VALUE!`. Engine differs on the written-empty cells (A1, A5); fix filed as T-2691. numbers.md updated.
