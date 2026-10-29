---
id: T-3088
title: "GROUPBY lifts over a multi-cell reference INDEX returns as total_depth"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GROUPBY lifts over a multi-cell reference that INDEX returns, given as `total_depth`, exactly as it lifts over a written
range, where ours gives one `#VALUE!`.

## Context
Found by T-2707 (arrays.md "GROUPBY and PIVOTBY lift over an array field_headers or total depth", the INDEX bullet;
probes/t2707-prob-orders, excel-graph 2026-10-29). Data!A4:D4 holds 2.5, 1E+15, "1", FALSE:
- Excel: `GROUPBY(..,0,INDEX(Data!A4:D4,0),-2)` (also `INDEX(Data!A4:D4,1,0)`, `INDEX(Data!A4:D4,<blank>)`) is 1x4 like
  `Data!A4:D4` (A50, A63:A65): ROWS 1, COLUMNS 4 (A37, A38, A49), ARRAYTOTEXT "#VALUE!, #VALUE!, -0.5, -0.5" (A39),
  ERROR.TYPE spills 3, 3, `#N/A`, `#N/A` (A51:D51), and PROB's data reads 4 cells against 1, `#N/A` (A43, A44, A60;
  gen-m9/17c110a53924beb8 Calc!A330 is the same call).
- Ours gives one `#VALUE!` for the INDEX reference (a written range and `INDEX(..)+0` already lift, A63, A68): the
  reference INDEX returns probably reaches `lift_options` as a value it does not take for a range.
- Code: `Model::lift_options` (functions/lookup_and_reference/lifted_options.rs). Check PIVOTBY's options and
  `field_headers` take the same path.

## Acceptance criteria
- [x] probes/t2707-prob-orders A37:A39, A43, A44, A49, A50, B51:D51, A60, A64 and A65 and gen-m9/17c110a53924beb8 Calc!A330
      pass and leave the baselines, with no new known failure.
- [x] arrays.md's INDEX bullet says ours matches.
- [x] `make check` green; reviewers approve.

## Out of scope
- PROB's own data order (T-2707).

## Outcome
`Model::lift_options` lifts an option whose expression returns a multi-cell reference (INDEX, OFFSET, ...) as it lifts
a written range: the new `Model::freeze` (lifted_options.rs; `frozen` now wraps it) hands back the reference it
evaluated to, so the option is read without evaluating it twice. GROUPBY and PIVOTBY share the path. Eval: 14 cells
fixed (probes/t2707-prob-orders A37:A39, A43, A44, A49, A50, B51:D51, A60, A64, A65; gen-m9/17c110a53924beb8 Calc!A330),
no regressions. Unit test `multi_cell_reference_from_a_function_lifts_like_a_written_range`; arrays.md's INDEX bullet
says ours matches.
