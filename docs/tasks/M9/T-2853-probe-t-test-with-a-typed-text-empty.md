---
id: T-2853
title: "Probe T.TEST with a typed text, empty text and boolean literal as a sample (T-2469 follow-up)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the T.TEST sample cases that T-2469 left UNVERIFIED (docs/excel-semantics/statistics.md, the T.TEST
error-order entry).

## Context
T-2469 made a text sample in T.TEST an array of one cell with no number, as COVARIANCE.P/.S do
(probes/t2436/computed-scalar). The only T.TEST oracle cell is a computed empty text:
`TTEST(IFERROR(#VALUE!,""),Data!B11:E11,1,3)` is `#DIV/0!` (gen-m9 46ac3ef5e61583ca A344). Ours now gives the same for a
typed literal and for a boolean sample, with no probe behind either.

## Acceptance criteria
- [ ] A probe case (excel-graph, canary-verified) with `T.TEST("",A1:A4,1,3)`, `T.TEST("a",A1:A4,1,1)`,
      `T.TEST(TRUE,A1:A4,1,3)`, `TTEST(FALSE,A1:A4,2,2)`, and the same with the sample as array2.
- [ ] statistics.md records each result; the engine follows, or a task is filed for each difference.
- [ ] `make check` green; reviewers approve.

## Out of scope
- COVARIANCE and COVAR (already probed).
