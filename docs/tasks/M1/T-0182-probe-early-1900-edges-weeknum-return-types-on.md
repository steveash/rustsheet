---
id: T-0182
title: "Probe early-1900 edges: WEEKNUM return types on serials 1-60, ddd/dddd of serial 0, EDATE/EOMONTH of serial 0"
milestone: M1
lane: research
status: todo
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
- [ ] Probe cells oracle-filled; findings in docs/excel-semantics/dates.md.
- [ ] Engine matches every probe cell (fix in this task if small, else file tasks).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
