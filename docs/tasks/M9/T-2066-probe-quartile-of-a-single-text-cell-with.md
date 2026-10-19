---
id: T-2066
title: "Probe QUARTILE of a single text cell with quart 7 and 1 (gen-m9 288b99797f58f149 A3)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
QUARTILE of a single text cell (" x ") with quart 7 and quart 1 has Excel-evidenced results, and the engine matches them.

## Context
gen-m9/288b99797f58f149 Calc!A3 `=BETAINV(B7,QUARTILE(D6,D5),B5)+..` (D6 = " x ", quart 7): Excel `#NUM!`, engine `#VALUE!`. BETAINV's invalid probability may decide it alone, so use the excel-probe skill on `QUARTILE(<cell " x ">,7)` and `QUARTILE(<cell " x ">,1)` directly. Code: functions/statistical/percentile.rs. Semantics: docs/excel-semantics/statistics.md "Array arguments given a scalar, a single cell or an error". Found by T-1403.

## Acceptance criteria
- [ ] The two probe cells (and `PERCENTILE.EXC(<no-number range>,1/0)` = `#DIV/0!`) are in eval/suites/probes with oracle values; gen-m9/288b99797f58f149 A3 passes or its cause is recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
