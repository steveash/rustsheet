---
id: T-0901
title: "Financial core (PV/FV/PMT/NPER/IPMT/PPMT/NPV) bit-exact parity"
milestone: M9
lane: functions
status: todo
priority: P1
size: M
depends_on: []
---

## Goal
Remove ULP-level and close mismatches in the annuity family; they likely share an evaluation-order root cause.

## Context
Gap report: PV 47, NPV 40, PPMT 34, PMT 26, NPER 26, IPMT 21, FV 20 failing cells.

## Acceptance criteria
- [ ] Known failures for these functions drop by ≥ 90% with no new failures; root cause documented in docs/excel-semantics/financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
IRR/XIRR/RATE solvers (separate task).
