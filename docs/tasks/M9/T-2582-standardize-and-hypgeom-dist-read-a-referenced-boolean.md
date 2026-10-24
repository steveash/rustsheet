---
id: T-2582
title: "STANDARDIZE and HYPGEOM.DIST read a referenced boolean as a number after T-1849: sd 0 is #NUM! before x's text (gen-m9, 3 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1849]
campaign: visible
---

## Goal
STANDARDIZE and HYPGEOM.DIST read a TRUE or FALSE from a cell as 1 or 0 (T-1849's rule), with STANDARDIZE's standard_dev check before x's text, as Excel does.

## Context
From T-2436's probe of the untraced gen-m9 cells. statistics.md "Booleans from a reference or a formula are numbers in the F, beta, chi-squared and hypergeometric functions" (T-1849 owns the reader change; this task adds these three cells once it lands and checks STANDARDIZE's order with a text x).

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): STANDARDIZE and HYPGEOM.DIST read a referenced FALSE as 0 (T-1849's rule): `STANDARDIZE(3,-1,Data!E12)` with E12 = FALSE and `STANDARDIZE(Data!B8,-2.5,Data!A6)` with B8 = "1.5" (text) and A6 = FALSE are `#NUM!` (standard_dev 0, checked before x's text; ours `#VALUE!`), and `HYPGEOM.DIST(-0.5,Data!E9,4,13,CELL("contents",Data!E$1))` with E9 = FALSE is `#NUM!` (ours `#VALUE!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 3 known-failure lines in eval/baseline/gen-m9.txt, 3 formulas.
Examples:
- gen-m9/530c636749a85209 Calc!A379 `=LOGNORMDIST(STANDARDIZE(3,MIN((-1),Data!$D7),Data!E12),Data!A9,0)`: Excel #NUM!, ours #VALUE!; inner `STANDARDIZE(3,MIN((-1),Data!$D7),Data!E12)` Excel #NUM!, ours #VALUE!
- gen-m9/ee2497a5e683c5d9 Calc!A128 `=CONFIDENCE.NORM(EDATE(52330,-6),LEN(REPT("x",255)),(STANDARDIZE(Data!$B8,(-2.5),Data!A6)^((-0.0001)-1e-15)))`: Excel #NUM!, ours #VALUE!; inner `STANDARDIZE(Data!$B8,(-2.5),Data!A6)` Excel #NUM!, ours #VALUE!
- gen-m9/d8266bc15533702a Calc!A316 `=HYPGEOM.DIST((-0.5),Data!E9,4,13,CELL("contents",Data!E$1))`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/530c636749a85209.case.json (1): Calc!A379
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A128
- gen-m9/d8266bc15533702a.case.json (1): Calc!A316

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): these 3 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The rule's quirks are in docs/excel-semantics/ with the probe cells as evidence; new probe cells are gated or baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells of the same functions owned by other tasks.
