---
id: T-1682
title: "Probe INDIRECT of defined-name text: values, a1=FALSE, sheet-qualified, scope and formula-valued names"
milestone: M8
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what `INDIRECT("<name>")` does with a defined name, and the engine matching it.

## Context
T-1657 made INDIRECT resolve a defined name that holds a cell or range reference
(`indirect_defined_name`, functions/lookup_and_reference/mod.rs), because Excel gives
`ISREF(INDIRECT("RCR"))` TRUE (probes/r1c1-name-use-rcr A2, excel-graph). Only that cell is
evidence; docs/excel-semantics/lookup.md "INDIRECT resolves a defined name that holds a reference"
lists the unprobed choices: the value (`SUM(INDIRECT("nm"))`), `a1` FALSE (ours: `#REF!`),
sheet-qualified `INDIRECT("Sheet1!nm")` (ours: `#REF!`), a sheet-scoped name shadowing a workbook one
(ours: sheet first), formula-valued and dynamic names (`=OFFSET(...)`, `=Sheet1!$A$1+1`; ours:
`#REF!`), and lowercase or mixed-case name text. Write one probe case (excel-probe skill) per
group, fill it with the oracle, and fix what differs.

## Acceptance criteria
- [ ] A probe case covers each choice listed in Context, oracle-filled; ours matches every cell or
  the gap is filed as a task.
- [ ] The lookup.md entry is VERIFIED for the probed choices.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
