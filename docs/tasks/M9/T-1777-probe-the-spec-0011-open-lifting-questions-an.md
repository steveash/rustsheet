---
id: T-1777
title: "Probe the SPEC-0011 open lifting questions (an error beside a refused range, 3-D ranges, the other Analysis ToolPak functions)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The questions SPEC-0011 leaves open are answered by excel-graph probes, recorded in docs/excel-semantics/arrays.md, and turned into table or kernel changes in follow-up tasks where Excel differs from the design's choice.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "Open questions". Write one probe case (excel-probe skill), tagged M9. Fill only it with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`. Questions:
1. An error argument left or right of a refused range: `EDATE(NA(),A1:A60)`, `EDATE(A1:A60,NA())`, `EFFECT(1/0,C1:C60)`, and the same next to an array. The design assumes the leftmost wins.
2. A 3-D range (`Sheet1:Sheet2!A1:A2`) in a `Lift` and in a `LiftArrays` argument. The design gives `#VALUE!`.
3. Whether the other Analysis ToolPak-era functions refuse a range: QUOTIENT, FACTDOUBLE, MULTINOMIAL, SERIESSUM, GCD/LCM (catalog `array`, check only), DOLLARDE/DOLLARFR (financial.md lists them; check a later argument).
4. FORMULATEXT and spill of a dynamic `=EDATE({45000;45031},A1:A60)`, as a hint to how Excel saves it.

## Acceptance criteria
- [ ] The probe case and its oracle sidecar are committed, and the findings are in arrays.md with cell citations.
- [ ] A follow-up task is filed for each question where Excel differs from the design's choice, and SPEC-0011's open-questions list is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the findings.
