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
- gen-m5 round-2 triage (T-2252, excel-graph, 2026-10-22) has two cells past the bound, both `#NUM!` in Excel:
  gen-m5/55afc805e00a1509 Calc!A101 `BASE(ROUNDUP(2^53,-6),10,5)` (9007199255000000; ours "9007199255000000") and
  gen-m5/08eafc9e61602bcb Calc!A98, BASE of about 7.47E+19 in radix 34 (ours "3TDTK1V8J6TPP"). numbers.md "BASE of zero with
  min_length 0 is empty; BASE past 2^53 is #NUM!" records them; the fix is T-2373, which waits for this probe's edge.

## Acceptance criteria
- [ ] A probes case with those cells, oracle-filled (excel-graph), in eval/suites/probes.
- [ ] numbers.md entry with the cut and the result past it (extend the entry named above); T-2373 is the fix task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing BASE (the fix task's).
