---
id: T-1715
title: "Probe bare R1C1-cell names in A1 mode and as LAMBDA parameters"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for three spellings ours handles without it: a bare `=R1C1` / `=R5C7*1` / `=SUM(R5C7)` outside any LET in an A1 workbook, a LAMBDA parameter spelled like an R1C1 cell (`=LAMBDA(R1C1,R1C1*2)(5)`, `=LAMBDA(R5C7,[R1C1],R5C7*2+ISOMITTED(R1C1))(5)`), and the same with FORMULATEXT of each.

## Context
T-1658 made an in-range uppercase `R<n>C<m>` a name in A1 mode (Excel takes it as a LET name, probes/r1c1-let-name-r1c1). Side effects with no oracle evidence, recorded as "Unprobed" in docs/excel-semantics/references.md "R1C1-shaped sheet and defined names": a bare `=R1C1` is now `#NAME?` in ours (was `#ERROR!`; `=r1c1` was already `#NAME?`), and LAMBDA parameters spelled so are accepted. Note Graph rejects some workbooks with bare R1C1-shaped tokens in a cell formula (same section); if it rejects these too, record that. Use the excel-probe skill; one name per case, like probes/r1c1-let-name-*.

## Acceptance criteria
- [ ] Probe cases with excel-graph sidecars (or a recorded Graph rejection) for the spellings above.
- [ ] references.md updated with the findings; a fix task filed for any mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
