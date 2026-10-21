---
id: T-2334
title: "IMCSC, IMLOG2 and IMCOT of 0 are #NUM!, not NaN/inf text"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IMCSC("0"), IMLOG2("0") and IMCOT("0") answer `#NUM!` as Excel does, not the text `NaNNaNi`, `-inf` and `inf`.

## Context
Found by T-2043's sweep: probes-numeric `im-last-digit-sweep` D21, M21, P21 (expected `#NUM!`; ours TypeMismatch). Contradicts the 'zero denominator stays #NUM!' note in docs/excel-semantics/engineering.md (COMPLEXs.xlsx V22:Y22 only pass because IMREAL/IMAGINARY turn the junk text into #NUM!). Check the neighbouring IM* functions at 0 for the same leak.

## Acceptance criteria
- [ ] D21:F21, M21:R21 of the sweep pass; a unit test covers the three calls.
- [ ] `make check` green; eval baseline updated for the cells that now pass; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit kernels (T-2044).
