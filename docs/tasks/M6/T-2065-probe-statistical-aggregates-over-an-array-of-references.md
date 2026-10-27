---
id: T-2065
title: "Probe statistical aggregates over an array of references, one formula per workbook (SPEC-0014 open questions)"
milestone: M6
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's results are known for MEDIAN, STDEV.S, VAR.S, LARGE, SMALL, COUNTIFS, AVERAGEIF, MAXIFS, SUMSQ, AVERAGEA, MAXA and CONCAT over a lifted OFFSET (an array of references).

## Context
SPEC-0014 "Open": two combined probe workbooks holding these (`MEDIAN(OFFSET(A1,{0;1;2},0))`, `STDEV.S(OFFSET(A1,{0;1},0))`, `LARGE(OFFSET(A1,{0;1},0),1)`, `SMALL(OFFSET(A1,0,0,{2;3}),2)`, `COUNTIFS(OFFSET(A1,{0;1},0),">0")`, `AVERAGEIF(OFFSET(A1,0,0,{1;2}),">0")`, `MAXIFS(..)`, `SUMSQ(..)`, `AVERAGEA(..)`, `MAXA(..)`, `CONCAT(..)`, data as in probes/reference-lift-consumers-b2) failed on excel-graph twice with "retries exhausted" at createSession. Write one case per formula (or bisect) so a workbook that Excel for the web cannot open is isolated; record what opens in docs/excel-semantics/lookup.md and SPEC-0014 rules 4-6, and note any formula that never opens as desktop-only (M14, decision 0039).

Evidence already in (T-2577, probes/t2577-indirect-ref-array A157, excel-graph 2026-10-26):
`STDEV(INDIRECT("B1",{1;1}))` is 0 (the references read through, B1 = 5 twice), ours `#VALUE!`; that known
failure is this task's to implement or hand on.

## Acceptance criteria
- [x] Each formula above has an oracle value or is recorded as one Excel for the web cannot open.
- [x] SPEC-0014's open list and T-2063/T-2064 are updated with the findings.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Twelve one-formula probes (probes/t2065-lift-*). Oracle values for nine (SPEC-0014, docs/excel-semantics/lookup.md);
MEDIAN, LARGE and SMALL failed to open on Excel for the web (two runs, 2026-10-26): T-1075 item 18 (M14).
Not implemented: per-element COUNTIFS, AVERAGEIF, MAXIFS go to T-2064; STDEV.S, VAR.S, SUMSQ to T-2063 (findings
appended to both). CONCAT's `#VALUE!` already matches ours. The separating probes (multi-cell elements for
STDEV.S/VAR.S/SUMSQ; a numeric element for AVERAGEA/MAXA) are a follow-up task.
