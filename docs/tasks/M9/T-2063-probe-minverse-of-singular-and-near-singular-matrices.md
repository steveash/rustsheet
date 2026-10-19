---
id: T-2063
title: "Probe MINVERSE of singular and near-singular matrices (cutoff or residual inverse)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's MINVERSE results for singular and near-singular matrices are known and recorded in
docs/excel-semantics/arrays.md.

## Context
T-1751 found that MDETERM uses partial-pivot LU with no singularity cutoff: `MDETERM({1,2,3;4,5,6;7,8,9})` =
6.661338147750939E-16 (arrays.md, "MDETERM: partial-pivot LU, no singularity cutoff"). The engine's MINVERSE
shares `lu_decompose` (crates/rustsheet-engine/src/functions/math_and_trigonometry/matrix.rs) but still gives
`#NUM!` when a pivot falls below f64::EPSILON (2.2E-16). That cutoff has no Excel evidence. Excel may instead
return a huge residual inverse (about 1E16 entries) for {1,2,3;4,5,6;7,8,9}, or `#NUM!` only for an exactly-zero
pivot. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with oracle values for MINVERSE (INDEX into each cell, or a spill) of {1,2,3;4,5,6;7,8,9},
      {1,2;2,4}, {1E-20}, {1E-17,0;0,1}, {1,1;1,1.0000000000000002} and {1E-300}.
- [ ] arrays.md records the rule. A follow-up implementation task is filed if the engine differs, or the fix
      goes in this PR when it is a one-line threshold change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MDETERM (T-1751).
