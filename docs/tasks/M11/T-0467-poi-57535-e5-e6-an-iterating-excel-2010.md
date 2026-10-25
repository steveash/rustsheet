---
id: T-0467
title: "poi/57535 E5:E6: an iterating Excel 2010 file's cached cycle is one sweep behind our recalculation; settle it with an excel-graph recalc"
milestone: M11
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The two visibly-off cells of `poi/57535.xlsx` are settled. Either Excel's own recalculation of the file
gives our values and the cells are scored against that evidence, or the difference is a real gap in our
iteration and is filed as an engine task with the cells.

## Context
Gap triage 2026-10-25 (`rustsheet-eval run --suite poi` on main c30bebf7). No task names these two
poi known failures (eval/baseline/poi.txt):
- `Sheet1!E5` `=0.1*(E4+E6)`: Excel cached 1.33333326, ours 1.333333326.
- `Sheet1!E6` `=E4+E5`: Excel cached 7.33333326, ours 7.333333326.
E4 is 6 (`=D4+1`), and the workbook sets `<calcPr calcId="145621" iterate="1"/>` (Excel 2010,
app version 14.0300), so iterateCount is 100 and iterateDelta 0.001 by default.

Under recalc.md's rules ("Iterative calculation sweeps the cycle in sheet order, in place" and
"iterateDelta stops the iterations for the whole workbook"), sweeping from 0 gives E5 0.6, 1.26, 1.326,
1.3326, 1.33326, ... (n-2 threes, then 26, at sweep n), and the change at sweep n is 6.6 * 10^(1-n). So:
- one calculation from 0 stops at sweep 5 (change 0.00066 < 0.001), E5 1.33326;
- each later calculation adds one sweep;
- the cached 1.33333326 is sweep 8, which is four calculations;
- ours, 1.333333326, is sweep 9: one sweep from the cached values, which the engine takes as the
  cycle's starting values.

So ours may simply be what Excel shows after one more recalculation of the saved file. Excel 2010
files have an old calcId, which makes current Excel recalculate them in full on open. If so, the cached
values are a stale state, not an engine gap, and scoring this case against them can never pass. That
reading is not checked: Excel may instead start a full recalculation from 0, or not iterate a converged
cycle again.

Evidence route: excel-graph is available (`cd tools && uv run rustsheet-oracle status`). poi keeps
oracle sidecars in eval/suites/poi/oracle (suite.toml `oracle_dir`), which supersede the cached values.
The eval adapter's three-calculation model for iterative probes is in crates/rustsheet-eval/src/engine.rs
(`calculations`, T-0429). Check how a sidecar for an iterating xlsx case is filled and scored, and how
many calculations that implies, before writing one.

## Acceptance criteria
- [ ] excel-graph's values for E5 and E6 after it recalculates `poi/57535.xlsx` are recorded in
      docs/excel-semantics/recalc.md, together with how many calculations the fill ran.
- [ ] If they equal ours, a `57535.oracle.json` sidecar is committed in eval/suites/poi/oracle, and the two
      lines leave eval/baseline/poi.txt with no new lines in any baseline. If they do not, an
      engine-lane task is filed with the observed values, and this task's Outcome links it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the iteration engine (crates/rustsheet-engine/src/iteration.rs) in this task.
