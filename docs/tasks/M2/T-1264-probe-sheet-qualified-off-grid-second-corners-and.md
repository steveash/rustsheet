---
id: T-1264
title: "Probe sheet-qualified off-grid second corners and absolute off-grid cells (Data!A1:B0, Data!$H$0)"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's value is known, and matched, for a sheet-qualified range whose second corner is off the grid
(`=Data!A1:B0`, `=Data!A1:B1048577`) and for an absolute off-grid cell (`=Data!$H$0`, `=$H$0`).

## Context
T-0296 made a sheet-qualified off-grid cell (`Data!H0`, `Data!A1048577`) lex as a sheet-scoped name
(`#NAME?`, probes/aggregate-range-function-num-grid). These neighbouring shapes still fail to parse
and show `#ERROR!`; docs/excel-semantics/references.md "A sheet-qualified cell off the grid is a name"
lists them as unprobed. Excel may read `B0` as a name joined by `:` (`#NAME?`), or reject the formula
(then the case records that it cannot be stored, as T-0139 did). Code: `consume_range` in
`expressions/lexer/mod.rs`, `consume_off_grid_cell_as_name` in `lexer/ranges.rs`.

## Acceptance criteria
- [ ] A probe case with these formulas has excel-graph values (or a note that Excel refuses the file).
- [ ] The lexer matches the evidence; references.md updated with the probe id.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Off-grid columns (`Data!XFE1`).
