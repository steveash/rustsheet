---
id: T-0182
title: "Probe early-1900 edges: WEEKNUM return types on serials 1-60, ddd/dddd of serial 0, EDATE/EOMONTH of serial 0"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the early-1900 edges next to T-0125, and the engine matches it.

## Context
From the T-0125 fidelity review (PR #181). Unverified: (1) `fn_weeknum` (functions/date_and_time.rs) anchors week 1 on a real 1900-01-01 but compares the stand-in date (one day early below serial 60), so non-Sunday return types may be off: probe `WEEKNUM(2,2)`, `WEEKNUM(9,2)`, `WEEKNUM(60,2)`, `WEEKNUM(1,21)`, `WEEKNUM(0)`. (2) `ddd`/`dddd` in formatter/format.rs still read the stand-in date, so `TEXT(0,"dddd")` is #VALUE! while `TEXT(0,"d")` is "0": probe `TEXT(0,"dddd")`, `TEXT(0.5,"ddd h:mm")`, `TEXT(60,"dddd")`. (3) `month_arithmetic_date` (formatter/dates.rs) treats serial 0 as #NUM!: probe `EDATE(0,1)`, `EOMONTH(0,0)`, `EOMONTH(0.5,1)`, and `EDATE(40000,2000000000)` (engine #NUM!, hand-written test). Add the cells to probes/dates-early-1900 or a new probe case (excel-probe skill).

## Acceptance criteria
- [x] Probe cells oracle-filled; findings in docs/excel-semantics/dates.md.
- [x] Engine matches every probe cell, or fixes are filed: not matched here; the fixes touch three code areas, so T-0995, T-0996 and T-0997 cover every failing cell.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe dates-early-1900-edges (44 formulas, excel-graph). Excel counts early-1900 weekdays from a Sunday serial 1: WEEKNUM(2,2) is 2, TEXT(0,"dddd") is Saturday, EDATE/EOMONTH accept serial 0 as day 0 of January 1900. Recorded in dates.md; engine tasks T-0995 (WEEKNUM), T-0996 (TEXT ddd/dddd), T-0997 (EDATE/EOMONTH). Eval delta: new-coverage failures only.
