---
id: T-1015
title: "Probe 3-D references in one-sheet spans and wrapped in IF/CHOOSE for INDEX, AREAS, MATCH, COUNTBLANK"
milestone: M1
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Probes record what Excel gives for AREAS, COUNTBLANK and INDEX with a one-sheet 3-D span, and for INDEX/MATCH with a 3-D span wrapped in IF or CHOOSE.

## Context
Follow-up of T-0197. Its tests assert the one-sheet-span cases without oracle evidence, and wrapped spans (`INDEX(IF(TRUE,Data1:Data3!A1:A2),1)`) go down the value path and give `#REF!` unprobed. Add cells to probes/3d-ref-functions and record findings in docs/excel-semantics/references.md.

## Acceptance criteria
- [ ] The probe cells have oracle values and the engine matches them or the gaps are baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
