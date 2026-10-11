---
id: T-1333
title: "Probe plain implicit intersection of a one-row range from a cell in its row (@Data!B2:C2)"
milestone: M9
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-0962]
---

## Goal
The general implicit-intersection rule for a one-row range seen from a cell in that row has plain-range evidence, not only `@XLOOKUP` evidence.

## Context
T-0962 fixed `implicit_intersection` (crates/rustsheet-engine/src/implicit_intersection.rs) on probes/xlookup-row-column-picks B2, C2, which reach it through `@XLOOKUP`. docs/excel-semantics/references.md "Implicit intersection of a one-row range by a cell in that row". Probe with the excel-probe skill: `=@Data!B2:C2` in another sheet's B2 and C2 (and E2, outside the columns), a legacy (`legacy_formulas`) `=Data!B2:C2` in the same cells, and a same-sheet layout that is not circular (e.g. `=@B10:C10` in a cell of column B whose row differs, versus one in row 10 outside B:C).

## Acceptance criteria
- [ ] The probe is filled by excel-graph and passes (or new failures get a task); references.md cites it and drops "unprobed".
- [ ] `make check` green; reviewers approve.

## Out of scope
-
