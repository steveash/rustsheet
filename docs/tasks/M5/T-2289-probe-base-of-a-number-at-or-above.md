---
id: T-2289
title: "Probe BASE of a number at or above 2^53 (the engine prints i64::MAX's digits)"
milestone: M5
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's BASE result for a number at or above 2^53 (and near 2^63) is known from oracle cells and recorded in docs/excel-semantics/numbers.md.

## Context
- Found by the T-2159 Excel-fidelity review: `base_text` (functions/math_and_trigonometry/mathematical.rs)
  truncates with `number.trunc() as i64`, which saturates, so `BASE(1E+300,2)` prints the 63 ones of
  i64::MAX. No oracle cell covers BASE's upper bound on `number`; Excel documents a limit of 2^53.
- Probe with the `excel-probe` skill: `BASE(2^53-1,2)`, `BASE(2^53,2)`, `BASE(2^53+2,16)`, `BASE(2^63,2)`,
  `BASE(1E+300,36)`, each with and without `min_length`.

## Acceptance criteria
- [ ] A probes case with those cells, oracle-filled (excel-graph), in eval/suites/probes.
- [ ] numbers.md entry with the cut and the result past it; a fix task filed if ours differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing BASE (the fix task's).
