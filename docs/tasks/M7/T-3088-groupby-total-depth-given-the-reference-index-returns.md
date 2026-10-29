---
id: T-3088
title: "GROUPBY total_depth given the reference INDEX returns is one #VALUE! outside PROB"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GROUPBY with a multi-cell reference returned by INDEX as `total_depth` gives Excel's results: one `#VALUE!` under
ROWS, COLUMNS, ISERROR, ERROR.TYPE, ARRAYTOTEXT and SUM, but a lifted array under PROB's data argument.

## Context
Found by T-2707 (arrays.md "GROUPBY and PIVOTBY lift over an array field_headers or total depth", the INDEX bullet;
probes/t2707-prob-orders, excel-graph 2026-10-29). Data!A4:D4 holds 2.5, 1E+15, "1", FALSE:
- `ROWS(ISERROR(GROUPBY(..,0,INDEX(Data!A4:D4,0),-2)))&"x"&COLUMNS(..)` is "1x1" (A64; `INDEX(Data!A4:D4,1,0)` A65 and
  `INDEX(Data!A4:D4,Data!C9)` with C9 blank A50 the same), while `Data!A4:D4` itself (A63) and `INDEX(Data!A4:D4,Data!C9)+0`
  (A68) lift to "1x4". `ERROR.TYPE(..)` is the one cell 3 (A51; ours spills B51:D51), `ARRAYTOTEXT(..)` is `#VALUE!` (A39),
  `IFERROR(ROWS(..),"err")` is "err" (A49), A37/A38 the same.
- Under PROB it lifts: `PROB(0,GROUPBY(..,INDEX(Data!A4:D4,Data!C9),-2),0)` is `#N/A` (1 cell against 4, A44; through `LET`
  A60; A43 and gen-m9/17c110a53924beb8 Calc!A330 are the same call), where ours is `#VALUE!`.
- So ours is the reverse of Excel: it lifts the INDEX reference under ROWS/ISERROR and gives one `#VALUE!` under PROB.
  First find what decides it in Excel (the caller's argument kind? PROB's data is an array argument, ROWS's a reference
  one): probe the same GROUPBY under other array-argument functions (SUMPRODUCT, COVAR, FREQUENCY) and under value
  arguments (ABS, N), and the reference from OFFSET or a `:` range operator instead of INDEX, before implementing.
- Code: `Model::lift_options` (functions/lookup_and_reference/lifted_options.rs).

## Acceptance criteria
- [ ] probes/t2707-prob-orders A37:A39, A43, A44, A49, A50, B51:D51, A60, A64 and A65 and gen-m9/17c110a53924beb8 Calc!A330
      pass and leave the baselines, with no new known failure.
- [ ] arrays.md states the rule with the new probe's evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PROB's own data order (T-2707).
