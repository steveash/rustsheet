---
id: T-0108
title: "1900 leap-year bug in DATE and serial conversions"
milestone: M1
lane: engine
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-0009]
---

## Goal
Serial 60 is 1900-02-29 as in Excel.

## Context
docs/excel-semantics/dates.md.

## Acceptance criteria
- [ ] probes/dates-and-blanks A1–A4 pass; ironcalc-calc DATE workbooks no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
DATE, DAY/MONTH/YEAR, text-date parsing and number formatting follow Excel serials 1..60 (probes/dates-and-blanks A1-A3 fixed; A4 already passed). `from_excel_date` stays a weekday-faithful stand-in; remaining early-1900 gaps are in T-0125.
