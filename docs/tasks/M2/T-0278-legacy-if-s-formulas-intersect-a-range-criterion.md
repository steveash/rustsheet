---
id: T-0278
title: "Legacy *IF(S) formulas intersect a range criterion (all but COUNTIF and AVERAGEIF return #VALUE!)"
milestone: M2
lane: functions-criteria
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
In a legacy (non-dynamic) formula every *IF(S) function intersects a multi-cell range criterion like COUNTIF and AVERAGEIF already do, instead of returning `#VALUE!`.

## Context
T-0263 fixture `fixtures/ifs-legacy-range-criteria.xlsx` (oracle excel-graph): with `A1:A6`/`D1:D2` criteria, formulas in rows 1 and 2 give the criterion `D1` / `D2`: COUNTIF, COUNTIFS, SUMIF (with and without a one-cell sum range), SUMIFS, AVERAGEIF, AVERAGEIFS, MAXIFS and MINIFS all intersect (3,3,7,7,7,2.33,2.33,3,1 in row 1). We already intersect COUNTIF (F) and AVERAGEIF (K); COUNTIFS (G), both SUMIFs (H, I), SUMIFS (J), AVERAGEIFS (L), MAXIFS (M) and MINIFS (N) return `#VALUE!` (see `make eval-show CASE=ifs-legacy-range-criteria`). COUNTIFS/SUMIFS/AVERAGEIFS/MAXIFS/MINIFS keep `Signature::Vector` criteria in `static_analysis.rs` (T-0207).

## Acceptance criteria
- [ ] All cells of `fixtures/ifs-legacy-range-criteria` pass; baseline shrinks.
- [ ] Dynamic-array behavior (lifting, probes/ifs-array-criteria) is unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing dynamic-array lifting of criteria.
