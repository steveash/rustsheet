---
id: T-0108
title: "1900 leap-year bug in DATE and serial conversions"
milestone: M1
lane: engine
status: todo
priority: P1
size: S
depends_on: []
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
