---
id: T-2584
title: "LINEST of a known_y range holding empty cells is #VALUE!, not a fit of the numbers (gen-m9, 1 cell)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-1741]
campaign: semantics
---

## Goal
LINEST of a known_y range that holds empty cells is `#VALUE!`, as Excel does, not a fit of its numbers.

## Context
From T-2436's probe of the untraced gen-m9 cells. statistics.md "LINEST, LOGEST and GROWTH in the gen-m9 sweep (T-2255)" (a range known_y with text is `#VALUE!`) and "LINEST: a known_y range with empty cells is #VALUE! (T-2436)". Check LOGEST, TREND and GROWTH (shared reader `read_regression_inputs` in functions/statistical/linest.rs) with the same probe. T-1741 is in flight on LINEST; coordinate.

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `LINEST(Data!E8:E12)` with E8:E10 = 1E-15, -2.5, -0.0001 and E11:E12 empty is `#VALUE!` (ours 0.25, a fit of the three numbers), so AVEDEV over its power is `#VALUE!` (ours `#NUM!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 1 known-failure line in eval/baseline/gen-m9.txt, 1 formula.
Examples:
- gen-m9/b8ee55934c5b824c Calc!A394 `=AVEDEV((LINEST(Data!E8:E12)^(Data!$B5-0.2)),Data!C3:C5,1e-15)`: Excel #VALUE!, ours #NUM!; inner `LINEST(Data!E8:E12)` Excel #VALUE!, ours 0.25
Cells:
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A394

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): this known-failure line disappears from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The rule's quirks are in docs/excel-semantics/ with the probe cells as evidence; new probe cells are gated or baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells of the same functions owned by other tasks.

## Outcome
Already fixed on `origin/main` before this task was worked: `probes/t2436/b8ee55934c5b824c` passes 15/15
and gen-m9 `b8ee55934c5b824c` Calc!A394 is not a known failure (it is absent from
`eval/baseline/gen-m9.txt`). `read_regression_inputs`' successor (`Grid::numbers` in
`functions/statistical/regression.rs`) already makes a blank in known_y `#VALUE!`. No code or baseline change;
eval delta 0. LOGEST, TREND and GROWTH share that reader and stay unprobed on Excel.
