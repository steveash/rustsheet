---
id: T-2643
title: "Probe convert-to-range for a defined name and out-of-table or cross-sheet #This Row references (T-2342)"
milestone: M8
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Evidence for what convert-to-range does to a defined name that refers to the table and to `#This Row` references the T-2342 probes did not cover.

## Context
T-2342 lowers defined-name references with the delete path's rewrite (unprobed) and probed only a `#This Row` beside a data row on the table's sheet. Probe a defined name `=Sales[Qty]` read via a cell, a `#This Row` outside the data rows, and one from another sheet (excel-probe skill); fix the engine where it differs.

## Acceptance criteria
- [ ] Probes with excel-graph oracle values added, references.md updated, engine matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
