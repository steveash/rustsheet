---
id: T-3533
title: "Probe MAXA's lifted text, MINA, VAR.P, STDEV.P, the VARA family and legacy VAR over a lifted OFFSET (T-2824 follow-up)"
milestone: M6
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2824]
---

## Goal
Excel's results are known for the aggregates T-2824 left on the value view, or read without evidence, over an array of
references (a lifted OFFSET), so each can follow Excel.

## Context
T-2824 made STDEV.S, VAR.S, SUMSQ, AVERAGEA and MAXA read each element of a lifted OFFSET by its top-left cell, on
probes/t2065-lift-* and probes/t2816-lift-* (docs/excel-semantics/lookup.md "Statistical aggregates over a lifted
OFFSET"). These are still open:
- MAXA reads that cell as an array element, which skips text. No probe tells skipped text from text read as 0:
  `MAXA(OFFSET(A4,{0;1},0))` with A4 "5" and A5 -3 gives 0 if text counts 0, and -3 if it is skipped.
- MINA, VAR.P, STDEV.P, VARA, VARPA, STDEVA and STDEVPA keep the value view (`#VALUE!`) and are unprobed.
- Legacy VAR shares VAR.S's reading, UNVERIFIED. Legacy STDEV is verified by probes/t2577-indirect-ref-array Calc!A157.
Write one formula per workbook, in E1, on the t2816 data (probes/t2816-lift-*).

## Acceptance criteria
- [ ] Probe cases filled on excel-graph for each function above, with single-cell, multi-cell, text, blank and TRUE
      elements; the finding is in lookup.md; the engine follow-up is filed, or the change is made if it is S-size.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MEDIAN, LARGE and SMALL (verified `#VALUE!`, lookup.md).
