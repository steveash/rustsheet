---
id: T-2065
title: "Probe statistical aggregates over an array of references, one formula per workbook (SPEC-0014 open questions)"
milestone: M6
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's results are known for MEDIAN, STDEV.S, VAR.S, LARGE, SMALL, COUNTIFS, AVERAGEIF, MAXIFS, SUMSQ, AVERAGEA, MAXA and CONCAT over a lifted OFFSET (an array of references).

## Context
SPEC-0014 "Open": two combined probe workbooks holding these (`MEDIAN(OFFSET(A1,{0;1;2},0))`, `STDEV.S(OFFSET(A1,{0;1},0))`, `LARGE(OFFSET(A1,{0;1},0),1)`, `SMALL(OFFSET(A1,0,0,{2;3}),2)`, `COUNTIFS(OFFSET(A1,{0;1},0),">0")`, `AVERAGEIF(OFFSET(A1,0,0,{1;2}),">0")`, `MAXIFS(..)`, `SUMSQ(..)`, `AVERAGEA(..)`, `MAXA(..)`, `CONCAT(..)`, data as in probes/reference-lift-consumers-b2) failed on excel-graph twice with "retries exhausted" at createSession. Write one case per formula (or bisect) so a workbook that Excel for the web cannot open is isolated; record what opens in docs/excel-semantics/lookup.md and SPEC-0014 rules 4-6, and note any formula that never opens as desktop-only (M14, decision 0039).

## Acceptance criteria
- [ ] Each formula above has an oracle value or is recorded as one Excel for the web cannot open.
- [ ] SPEC-0014's open list and T-2063/T-2064 are updated with the findings.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
