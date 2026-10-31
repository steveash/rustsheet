---
id: T-3526
title: "GEOMEAN in-range root: the cells the split root misses (geomean-root-refinement A23, A33, A34, A44 and the rest)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
defer: phase-1b
size: M
tier: opus
depends_on: []
---

## Goal
GEOMEAN's in-range root matches Excel's bits on the cells the T-3502 split root misses, or the task records why no
model in doubles can.

## Context
- T-3502 replaced the 15-digit snap with the split root `exp(ln(m·2^r)/n)·2^q` (numbers.md "GEOMEAN in-range
  root", decision 0211). It misses 11 of 180 fresh cells (probes-numeric/t3502-geomean-fresh), 8 of 180 post-freeze cells
  (t3502-geomean-postfreeze A5, A47, A50, A70, A74, A79, A99, A115), 7 of the 69
  literal in-range probe cells, in-range-many A2 and Copies A35, and moved four probes cells one ulp off the bit
  under decision 0211: geomean-root-refinement Sheet1!A23, A33, A34, A44 (there Excel gives `exp(ln(P)/n)`).
- Excel's root behaves like a library root of up to about 0.8 ulp; earlier models are listed in numbers.md.
- The fit lab has the candidates (`rustsheet-fitlab geomean-snap`, `geomean-snap-fresh`).
- Phase 1b under decision 0188 R5 unless a model fixes a visibly-off cell: every miss is within 1 ulp.

## Acceptance criteria
- [ ] A model that keeps every cell the split root matches and fixes some of the misses, with the fit lab's
      hold-out counts and a fresh sweep (decision 0188 R4), or an Outcome listing the models tried and their
      counts.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GEOMEAN's out-of-range path (T-1099's root rounding).
