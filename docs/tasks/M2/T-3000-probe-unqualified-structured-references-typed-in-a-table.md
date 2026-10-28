---
id: T-3000
title: "Probe unqualified structured references typed in a table's totals and header rows, and the printed form of a typed [[#This Row],[Col]] (T-2979)"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for three engine choices T-2979 made without a probe, recorded in references.md ("Structured
references: this row, totals and ranges", the "Ours (T-2979)" note), with an implementation task filed if the
engine differs.

## Context
T-2979 resolves an operand starting with `[` in a table cell to that table. Unprobed: (1) a totals-row cell counts as
inside the table (`=SUBTOTAL(109,[Col])`, `=[@Col]` typed in the totals row: value and FORMULATEXT); (2) a header-row
cell counts too (Excel may store a header formula as text, references.md "Calculated columns" rule 7); (3) a typed
`=[[#This Row],[Col]]` prints as `=[@Col]` (engine choice; the probed `=[@Col]*2` prints as typed,
t2957-fill-one-totals F2:F4). Also worth one cell each: the same `=[@Col]*2` typed in two tables of one sheet, and
`=[@Col]` typed outside every table. Use the `excel-probe` skill with `entered` cells typed after the tables, as the
t2957-* cases do.

## Acceptance criteria
- [ ] A probes case with oracle values for (1)-(3) and the two extra cells.
- [ ] The finding recorded in references.md; an implementation task filed if the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
