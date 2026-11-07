---
id: T-4801
title: "Probe RANK, RANK.EQ and RANK.AVG over a lifted OFFSET or INDIRECT ref (T-4740 follow-up)"
milestone: M6
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's values for RANK, RANK.EQ and RANK.AVG whose `ref` is an array of references from a lifted OFFSET or INDIRECT
(`RANK(5,OFFSET(A1,{0;1},0,2))`, `RANK(5,INDIRECT({"A1:A2";"B1:B2"}))`) are known, and ours matches or a fix task exists.

## Context
- T-4740 made the RANK family lift per element over any array of references (`rank_per_element`,
  statistical/rank_eq_avg.rs): a reference element ranks against its own range. The evidence is the mixed XLOOKUP
  grid only (probes/t4704-mixed-xlookup-grid-readers A25:C38). Before T-4740 a lifted OFFSET or INDIRECT `ref` was
  read as its value view, `#VALUE!` per element. No probe covers either reading.
- Probe with the `excel-probe` skill, predictions committed before the fill: OFFSET and INDIRECT grids of one-cell
  and multi-cell references, one with an error element (INDIRECT of a bad address), a lifted `number` beside the grid,
  and ROWS/COLUMNS of each. Use different values in each referenced range, so per-element ranking can be told apart
  from the value view.
- Probe both sheet axes: grids (XLOOKUP, OFFSET, INDIRECT) whose references are on the formula's own sheet and on
  another. LARGE, SMALL and MEDIAN split on that axis (`#N/A` or the value view; probes/t4569 Local!F1:F9, SPEC-0014
  rule 5), and RANK may too; t4704's evidence is cross-sheet only. Add an error `order` beside a reference element
  (reviewer-excel-fidelity on #3874).
- SPEC-0014 amendment "As implemented (T-4570)", RANK family bullet.

## Acceptance criteria
- [ ] A probe case filled on excel-graph covers the shapes above; the finding is in docs/excel-semantics/lookup.md and
      statistics.md ("RANK of a non-reference array"), which until then mark the per-element reading of these grids
      UNVERIFIED.
- [ ] Ours matches every cell, or each mismatch has a fix task naming its cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- LARGE and SMALL over the same grids (SPEC-0014 rule 5 covers them).

## Outcome
Probe `probes/t4801-rank-lifted-offset-indirect` (27 shapes, same-sheet and off-sheet, 217 cells) filled on excel-graph.
Excel ranks each element of a lifted OFFSET/INDIRECT grid against its own range, identical on both sheet axes, errors
element-wise; the per-element reading held, but the committed probe-3 prediction ({2;3}) was wrong (Excel and ours give {2;2}: 5 ranks 2nd in B1:B2 = 9,5). No data range has a tie, so RANK.AVG averaging over a lifted grid is not exercised (follow-up probe). Ours matches all 217 cells, so no fix task. Findings in docs/excel-semantics/lookup.md and
statistics.md. Eval delta: +217 passing probe cells, no regressions.
