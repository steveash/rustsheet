---
id: T-0108
title: "1900 leap-year bug in DATE and serial conversions"
milestone: M1
lane: engine
status: blocked
priority: P1
size: S
depends_on: [T-0004]
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

## Blocked because
Needs the Excel oracle (T-0004) — owner must provide Graph credentials (docs/architecture/oracle.md).
