---
id: T-3744
title: "Probe ROUND past a 16-bit num_digits as a control for the ROUNDUP/ROUNDDOWN 100-place step; cross-reference the wrap entry"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3627]
---

## Goal
numbers.md says from oracle evidence whether ROUND shows the 100-place step that ROUNDUP and ROUNDDOWN take past a 16-bit
`num_digits`, and its huge-`num_digits` entries point at each other.

## Context
- Deferred MINOR findings of the excel-fidelity and quality reviews of PR #2705 (T-3627). T-3627 found that ROUNDUP and
  ROUNDDOWN round at 100 places once the narrowed |k| >= 32,768 (numbers.md, "ROUNDUP and ROUNDDOWN round at 100 places
  past a 16-bit num_digits"). It says "ROUND's narrowing differs and shows no such step", but no ROUND cell was probed
  with |k| in [32,768, 65,535] and digits past 10^±100; T-3635's out-of-scope line relies on that sentence.
- Probe with the `excel-probe` skill: `ROUND(1.23456789012345E-95,40000)`, `ROUND(1.23456789012345E+105,-32768)`,
  `ROUND(1.5E-120,65535)`, `ROUND(1.5E+120,-65535)` and the same at k = ±32,767, plus TRUNC at the same cells (T-3635
  warns TRUNC shares the ROUNDDOWN path and was not probed past 32,767).
- Docs nits from the same reviews: numbers.md "ROUND family: huge num_digits wrap" needs a "see also" line to the new
  section; docs/tasks/M0/T-3555-gate-a-slice-3e-reference-tables-for-round.md cites the renamed heading "ROUNDUP of a
  huge value at a huge negative num_digits"; the description of
  eval/suites/probes/cases/t3627-roundup-rounddown-huge-digits-place.case.json names a case
  "t3555-roundup-rounddown-digits-by-exponent" that is t3627-roundup-rounddown-digits-by-exponent (changing it changes
  the case's xlsx input hash: re-fill the oracle or leave it, and say which); T-3635 has an empty "Out of scope".

## Acceptance criteria
- [ ] A probe case (excel-graph) with the ROUND and TRUNC cells above; numbers.md states ROUND's and TRUNC's behaviour
      past 32,767 (VERIFIED), and if either shows the step, an engine task is filed or T-3635 is extended.
- [ ] The docs nits above are fixed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The ROUNDUP/ROUNDDOWN engine change (T-3635).
