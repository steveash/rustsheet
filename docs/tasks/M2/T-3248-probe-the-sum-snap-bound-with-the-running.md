---
id: T-3248
title: "Probe the SUM snap bound with the running total below the last addend's exponent"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence, inside SUM, that the near-zero snap bound is 8 ULP of the running total before the last addend and
not of the larger of the two final addends.

## Context
- T-3145 (probes/t3145-snap-multi-addend; docs/excel-semantics/numbers.md "Near-zero snapping", "Three or more
  addends") showed 3+ addend SUMs snap by the running total, not the first argument or the last addend. In each of
  its blocks the running total is the larger final addend or shares its exponent, so within SUM it does not
  separate "running total" from "larger addend"; only the two-addend `-` operator evidence (T-3100) does.
- Probe form (excel-fidelity review of PR #2423): `SUM(0.5, 0.5-k*2^-52, -1)` and similar for k = 1..16, where the
  running total 1-k*2^-52 (exponent -1, bound 2^-50: snaps k ≤ 3) sits below the last addend 1 (exponent 0, bound
  2^-49: k ≤ 7). Also over a range, an array argument, SUBTOTAL(9) and the `+` chain.
- Engine: `SumParts::snapped` (functions/math_and_trigonometry/sum.rs) uses the running total.

## Acceptance criteria
- [x] An excel-graph probe (milestone M2) with those cells; numbers.md states the result.
- [x] If the engine differs, `SumParts::snapped` matches Excel, with tests either side of the bound.
- [x] `make check` green; reviewers approve.

## Out of scope
- The opposite-sign condition.

## Outcome
- probes/t3248-snap-running-total-below-last (384 cells, excel-graph 2026-10-30): inside SUM the near-zero snap bound
  is 8 ULP of the running total before the last addend, not of the larger final addend. With the running total
  1-k·2^-52 below the last addend ∓1, all four blocks snap exactly k ≤ 3 and keep k = 4 in SUM over arguments, a
  range and an array argument, SUBTOTAL(9) and the `+` chain; AVERAGE snaps before dividing.
- numbers.md "Near-zero snapping" records it (VERIFIED). The engine already matched (`SumParts::snapped`), so no
  code change; eval delta +384 passing probe cells, no new known failures.
