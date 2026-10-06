---
id: T-0263
title: "Probe *IFS legacy range criteria and lifted-criteria edge cases"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the *IF(S) array-criteria cases T-0207 left unprobed, recorded in
docs/excel-semantics/criteria.md ("*IF(S) lift array criteria").

## Context
T-0207 review (excel-fidelity). (1) Legacy formulas: COUNTIF/AVERAGEIF intersect a range criterion
(poi/FormulaEvalTestData_Copy.xlsx EverythingTests!F296), but COUNTIFS/SUMIFS/AVERAGEIFS/MAXIFS/MINIFS
keep `Signature::Vector` criteria (static_analysis.rs), so a legacy `COUNTIFS(A:A,D1:D2)` is not
intersected; also EverythingTests!E1324 `SUMIF(AA7:AA13,C10:D10,C7)` expects 1.1. Needs a fixture
xlsx with non-dynamic formulas (gated `fixtures` suite). (2) Dynamic cases to add to
probes/ifs-array-criteria: a SUMIFS shape mismatch with an array criterion (`SUMIFS(B1:B5,A1:A6,{"a","b"})`:
one #VALUE! or an array of them?), a range criterion with a blank cell, an error first argument with
an array criterion.

## Acceptance criteria
- [ ] Probe cells for (1) and (2) are oracle-filled (excel-graph).
- [ ] criteria.md records each finding; an implementation task is filed for any engine mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing fixes.
