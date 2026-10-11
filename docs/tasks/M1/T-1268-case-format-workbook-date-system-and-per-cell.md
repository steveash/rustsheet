---
id: T-1268
title: "Case format: workbook date_system and per-cell number formats; probe 1904 negatives (SPEC-0005 step 2)"
milestone: M1
lane: harness
status: in-progress
priority: P3
size: S
tier: opus
depends_on: [T-1267]
spec: docs/specs/SPEC-0005-workbook-date-system.md
---

## Goal
Probe cases can request a 1904 workbook and set a cell's number format, and the SPEC-0005
unknowns have excel-graph values.

## Context
SPEC-0005 "Excel behavior to match" (unknowns) and rollout step 2. The `rustsheet-case/v1` format
can set neither (T-1105 Context). The spec→xlsx builder in `tools/` writes the case workbook;
the engine side reads the flag through T-1267.

## Acceptance criteria
- [ ] Optional case fields for `date_system` and per-cell `number_format`, documented, with builder and loader tests.
- [ ] A probe case for `TEXT(-1.5,"[h] d")`, `TEXT(-1E-10,"[h]:mm")`, `TEXT(-0,"[h]:mm")` in both systems and cell display of a negative under a date format, filled by excel-graph; findings in dates.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine behavior changes (T-1105).
