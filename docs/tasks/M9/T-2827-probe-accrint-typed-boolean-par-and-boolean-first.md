---
id: T-2827
title: "Probe ACCRINT typed boolean par and boolean first_interest, settlement, frequency, basis"
milestone: M9
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
ACCRINT's behavior for a typed boolean par, and for a boolean first_interest, settlement, frequency and basis, is probed on Excel and matched.

## Context
T-2792 made `accrint_of` read par with `cast_number_no_bools_ref`: a boolean par is `#VALUE!` from a cell (gen-m9/932b33b1ef8b7608 A233) and, since the engine cannot tell them apart, typed too (unprobed). Booleans in the other arguments still read as numbers (unprobed). See docs/excel-semantics/financial.md "ACCRINT check order"; use the excel-probe skill.

## Acceptance criteria
- [ ] A probe case (typed and cell booleans in par, first_interest, settlement, frequency, basis) has oracle values; the engine matches them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ACCRINT's values.
