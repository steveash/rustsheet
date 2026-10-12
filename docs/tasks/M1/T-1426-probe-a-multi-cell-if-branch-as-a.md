---
id: T-1426
title: "Probe a multi-cell IF branch as a VLOOKUP/HLOOKUP index (follow-up of T-1377)"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's value for a multi-cell `IF` branch as the VLOOKUP/HLOOKUP index is recorded: `COUNT(HLOOKUP(42,F5:G6,IF(TRUE,H1:H2)))` and a value-only form that tells implicit intersection from lifting.

## Context
T-1377 made an `IF` index a reference (`lookup_index_arg`, index_arg.rs), so `IF(TRUE,H1:H2)` now behaves like the plain `H1:H2` index and `IF(TRUE,(G5,G6))` is `#VALUE!`. Neither has oracle evidence; docs/excel-semantics/references.md marks the plain multi-cell index unprobed. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe case added under eval/suites/probes with oracle values; findings recorded in docs/excel-semantics/references.md; engine mismatches filed as tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
