---
id: T-1944
title: "Probe legacy @ on SWITCH returning a range or nested under scalar functions"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's FORMULATEXT and values are known for a legacy cell with a SWITCH returning a range, or nested under a scalar function or operator, and ours matches.

## Context
T-1910 drops the legacy `@` only from a formula-root SWITCH returning array constants (`is_root_switch_of_array_constants`, static_analysis/legacy.rs); nested or range-returning SWITCH keep the `@` as before. Unprobed: `=SWITCH(1,1,A1:A3)` off and on row, `=SWITCH(1,1,A1:B2)`, `=ABS(SWITCH(1,1,{-2;3}))`, `=-SWITCH(1,1,{2;3})`, `=LEN(SWITCH(1,1,{"ab";"c"}))`. Use the excel-probe skill (legacy cells with FORMULATEXT twins).

## Acceptance criteria
- [ ] A probes case covers the shapes above with oracle values; the parser matches them (baseline shrinks or the findings are recorded in references.md).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
