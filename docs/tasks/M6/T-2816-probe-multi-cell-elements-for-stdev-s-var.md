---
id: T-2816
title: "Probe multi-cell elements for STDEV.S, VAR.S, SUMSQ and a numeric element for AVERAGEA, MAXA over a lifted OFFSET"
milestone: M6
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's values are known for STDEV.S, VAR.S and SUMSQ over a lifted OFFSET whose elements are multi-cell ranges, and for AVERAGEA and MAXA over elements that include a number.

## Context
T-2065 probed STDEV.S, VAR.S, SUMSQ with single-cell elements only (top-left cell versus whole element is not separated), and AVERAGEA/MAXA over "5", blank, TRUE, which gave 0 and settle little. Suggested probes: `STDEV.S(OFFSET(A1,0,0,{1;2}))` (top-left gives `#DIV/0!`, whole element the STDEV.S of {1,1,100}); `AVERAGEA(OFFSET(A4,{0;3},0))` over "5" and 7 (3.5, 7 or 0) and `AVERAGEA(OFFSET(A5,{0;1},0))`, and MAXA the same. One formula per workbook (docs/specs/SPEC-0014-reference-array-results.md, docs/excel-semantics/lookup.md).

## Acceptance criteria
- [ ] Each probe has an oracle value; lookup.md and SPEC-0014 state the rules, and T-2063 is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
