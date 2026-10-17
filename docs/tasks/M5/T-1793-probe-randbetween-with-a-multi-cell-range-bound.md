---
id: T-1793
title: "Probe RANDBETWEEN with a multi-cell range bound beside an error bound, both positions"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence settles whether Excel's top-first error order (T-1782) also applies when a multi-cell range bound sits beside an error bound, and the engine matches it.

## Context
T-1782 review (fidelity, MINOR): `fn_randbetween` (functions/math_and_trigonometry/random.rs) checks for a multi-cell range bound and omitted bounds before the top-first read, so `RANDBETWEEN(D1:D3,NA())` and `RANDBETWEEN(NA(),D1:D3)` are both `#VALUE!` in the engine; no probe backs that. Use scalar-returning wrappers (ERROR.TYPE over INDEX). Record in docs/excel-semantics/numbers.md beside the T-1782 entry.

## Acceptance criteria
- [ ] A probe (excel-graph) covers a multi-cell range beside `NA()` / `#DIV/0!` / text in both positions.
- [ ] The engine matches every cell; numbers.md records the rule.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
