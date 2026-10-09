---
id: T-1173
title: "Probe grouped defined-name bodies in reference positions (OFFSET, AREAS, range operator, INDEX, CELL)"
milestone: M1
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence and engine parity for a defined-name body written wholly in parentheses used in reference positions: `OFFSET(rg,1,0,1,1)`, `AREAS(rg)`, `SUM(ig:B2)`, `ROWS(rg:B3)`, `INDEX(rg,2)` and `CELL("address",rg)`.

## Context
T-1164 made name bodies keep a `GroupedRootKind` root, so such names are `Formula(GroupedRootKind(..))` instead of the bare node. ISREF, ROWS and SUM are covered by probes/defined-name-grouped-reference (excel-graph); the reference-position uses above are unprobed and reference_operators.rs (`range_operand`, `eval_ref_or_index`) may not look through the grouped root.

## Acceptance criteria
- [ ] Cells for the six forms added to probes/defined-name-grouped-reference with an excel-graph sidecar (re-run `rustsheet-oracle fill`; the case hash changes).
- [ ] The engine matches (look through `GroupedRootKind` where it does not); baseline consistent.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Conditional-formatting formulas (T-1171).
