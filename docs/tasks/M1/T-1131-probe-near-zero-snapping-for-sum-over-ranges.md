---
id: T-1131
title: "Probe near-zero snapping for SUM over ranges, trailing zeros, arrays and =(SUM(...))"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for near-zero snapping of an outermost SUM in the forms T-0110 extrapolated, and the engine matching it.

## Context
T-0110 snaps the final addition of an outermost SUM (`fn_sum_outermost`, functions/math_and_trigonometry/sum.rs; `snap_near_zero`, arithmetic.rs) using the last two addends. The evidence covers only `SUM(X,-b)` (probes-numeric/30bc5f8fab02d0a2 `sum-fn`) and `SUM(0.1,0.2,-0.3)` (probes/precision-and-rounding A4). Unprobed (docs/excel-semantics/numbers.md "Near-zero snapping"): `=SUM(A1:A3)` over 0.1, 0.2, -0.3 (also with blanks or text between); `=SUM(0.1,0.2,-0.3,0)`; `=SUM({0.1,0.2,-0.3})`; `=(SUM(0.1,0.2,-0.3))`; and the order of addition in a 2-D range (row-major?). Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe cells with oracle values for each form; the numbers.md Unprobed line is resolved; the engine matches (baseline shrinks or a DEVIATION entry).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
