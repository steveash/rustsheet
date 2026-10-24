---
id: T-2615
title: "Probe PRICEMAT error order (yld vs basis) and HLOOKUP with a text-cell range_lookup inside LOGEST"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence decides two gen-m9 cells T-1742 could not explain from LOGEST: PRICEMAT's choice between an error yld and
an error basis, and HLOOKUP with a range_lookup that is a reference to a text cell "TRUE", as known_x of LOGEST.

## Context
- gen-m9 bc04c1f478c9896c Calc!A75 `=PRICEMAT("2024-02-29",46852,34604,0.3,CONFIDENCE.NORM(1.5,COLUMN(Data!$A11),13),
  (2/LOGEST(Data!C$3,Data!E8:E12,Data!$D$6,Data!B9)))` is `#VALUE!` in Excel, `#NUM!` in ours. The yld is CONFIDENCE.NORM's
  `#NUM!` (alpha 1.5); the basis is `2/LOGEST(..)`, and ours gives that LOGEST `#VALUE!` (D6 holds the text "#N/A" as
  const). So Excel either reports the basis error before the yld error, or its LOGEST there is not an error (an array
  basis). Probe `PRICEMAT(<valid dates>,0.3,NA(),1/0)`, `PRICEMAT(..,0.3,1/0,"x")`, the cell's LOGEST alone, and
  `ROWS(LOGEST(Data!C$3,Data!E8:E12,Data!$D$6,Data!B9))` with that data.
- gen-m9 b8ee55934c5b824c Calc!A161 `=LOGEST(Data!A7:D9,HLOOKUP((42<>Data!E3),Data!A9:D10,(2+Data!C3),INDIRECT("Data!A1")))`
  is `#VALUE!` in Excel, `#REF!` in ours. Data!A1 is the text "TRUE"; ours reads it as TRUE, finds TRUE at C9 and returns
  the empty C10 (a 1x1 known_x against a 3x4 known_y: the size `#REF!`). A direct blank one-cell known_x is `#REF!` in
  Excel too (`LOGEST(Data!B9:D12,Data!A$9)`, 05d44f6dd6cfc567 A30, and 7 more cells), so either HLOOKUP is `#VALUE!` for
  a range_lookup that refers to a text cell (lookup.md shows only typed "TRUE" working, G18), or a computed blank
  known_x is `#VALUE!`. Probe `HLOOKUP(TRUE,<row>,2,<ref to "TRUE">)`, `ISERROR` of it, and `LOGEST({1;2;4},IF(TRUE,
  <empty cell>))` against `LOGEST({1;2;4},<empty cell>)`.
Recorded in statistics.md "LINEST, LOGEST and GROWTH: cells left after T-1741 (T-1742)".

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with oracle values for each bullet under Context is committed under eval/suites/probes.
- [ ] financial.md, lookup.md or statistics.md record the findings, and a functions task is filed for each mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine.
