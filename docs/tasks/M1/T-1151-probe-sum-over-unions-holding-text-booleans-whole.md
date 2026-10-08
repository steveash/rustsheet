---
id: T-1151
title: "Probe SUM over unions holding text, booleans, whole columns and errors (follow-up of T-1149)"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence that SUM over a union treats text, booleans, whole-column areas and error cells as it does for separate arguments.

## Context
T-1149's metamorphic test assumes the relation also holds for text/boolean cells and whole columns inside a union, and for error precedence across areas; probes/multiarea-agg only covers numbers. See docs/excel-semantics/references.md "Multi-area references in functions".

## Acceptance criteria
- [ ] Probe rows (e.g. `SUM((A1:A3,B:B))` over text/boolean cells; two error cells in different areas) with oracle values, and the finding recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (file them if the probe disagrees with the engine).
