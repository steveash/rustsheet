---
id: T-1166
title: "ISREF is FALSE for a range whose corners are on two different sheets"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`=ISREF(Data!A1:Other!B2)` is FALSE, as in Excel; the cross-sheet range already gives `#VALUE!`.

## Context
Found by T-1138: probes/sheet-qualified-corner-order Calc!A7 (oracle excel-graph, 2026-10-08) expects FALSE, the engine returns TRUE. The other cross-sheet forms (SUM, `Data!A1:Other!A1`, IFERROR) already match. Code path: `get_range` in crates/rustsheet-engine/src/reference_operators.rs and the ISREF implementation.

## Acceptance criteria
- [ ] probes/sheet-qualified-corner-order Calc!A7 passes and is removed from eval/baseline/probes.txt via `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the `#VALUE!` result of the cross-sheet range itself.
