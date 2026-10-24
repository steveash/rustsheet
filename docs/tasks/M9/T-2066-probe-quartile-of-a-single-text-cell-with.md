---
id: T-2066
title: "Probe QUARTILE of a single text cell with quart 7 and 1 (gen-m9 288b99797f58f149 A3)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
QUARTILE of a single text cell (" x ") with quart 7 and quart 1 has Excel-evidenced results, and the engine matches them.

## Context
gen-m9/288b99797f58f149 Calc!A3 `=BETAINV(B7,QUARTILE(D6,D5),B5)+..` (D6 = " x ", quart 7): Excel `#NUM!`, engine `#VALUE!`. BETAINV's invalid probability may decide it alone, so use the excel-probe skill on `QUARTILE(<cell " x ">,7)` and `QUARTILE(<cell " x ">,1)` directly. Code: functions/statistical/percentile.rs. Semantics: docs/excel-semantics/statistics.md "Array arguments given a scalar, a single cell or an error". Found by T-1403.

gen-m9 sweep (gap triage 2026-10-23, T-2255). This task's cell, now in the gated gen-m9 baseline (the sweep gated the suite).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas.
Examples:
- gen-m9/288b99797f58f149 Calc!A3 `=(BETAINV(Data!$B$7,QUARTILE(Data!$D6,Data!D5),Data!B5)+MIN(Data!C2))`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/288b99797f58f149.case.json (1): Calc!A3

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The two probe cells (and `PERCENTILE.EXC(<no-number range>,1/0)` = `#DIV/0!`) are in eval/suites/probes with oracle values; gen-m9/288b99797f58f149 A3 passes or its cause is recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): the gen-m9 cells listed under Context are visibly off (a wrong type, error or
text, or a number further than half a unit in the 15th significant digit from Excel's), so the task is in the campaign. Its
target is that every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states
it. Bit-level criteria still unmet then move to a follow-up task without the key.

## Outcome
A single cell holding text is now empty data for PERCENTILE*/QUARTILE*/PERCENTRANK* (`#NUM!`, not `#VALUE!`); probe quartile-single-text-cell added with excel-graph values; gen-m9/288b99797f58f149 A3 passes.
