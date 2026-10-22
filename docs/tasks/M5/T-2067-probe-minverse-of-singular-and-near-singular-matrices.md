---
id: T-2067
title: "Probe MINVERSE of singular and near-singular matrices (cutoff or residual inverse)"
milestone: M5
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's MINVERSE results for singular and near-singular matrices are known and recorded in
docs/excel-semantics/arrays.md.

## Context
- Milestone M9 -> M5 (gap triage 2026-10-20): this task owns gen-m5 known failures of an M5 catalog function, and decision 0085 item 3 requires every failing cell of an M5 exit criterion to be owned by a task with milestone M5 so the pick order reaches it while M5 is current. Id and scope are unchanged.

T-1751 found that MDETERM uses partial-pivot LU with no singularity cutoff: `MDETERM({1,2,3;4,5,6;7,8,9})` =
6.661338147750939E-16 (arrays.md, "MDETERM: partial-pivot LU, no singularity cutoff"). The engine's MINVERSE
shares `lu_decompose` (crates/rustsheet-engine/src/functions/math_and_trigonometry/matrix.rs) but still gives
`#NUM!` when a pivot falls below f64::EPSILON (2.2E-16). That cutoff has no Excel evidence. Excel may instead
return a huge residual inverse (about 1E16 entries) for {1,2,3;4,5,6;7,8,9}, or `#NUM!` only for an exactly-zero
pivot. Use the `excel-probe` skill.

- gen-m5 (gap triage 2026-10-20, T-2114, excel-graph): 1 more cell, in eval/baseline/gen-m5.txt. `MINVERSE(1.0000000000000085E-108)` (a 1x1 matrix, `1E-15^(0.2+7)`) is 9.999999999999914E+107 in Excel; ours `#NUM!` from the 2.2E-16 pivot cutoff. So a 1x1 pivot far below the cutoff is inverted (recorded in arrays.md "MDETERM: partial-pivot LU, no singularity cutoff"). Acceptance addition: this line leaves eval/baseline/gen-m5.txt.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/9284730052aff46c Calc!A39 | `=MINVERSE((1e-15^(Data!$B7+Data!B12)))` | 9.999999999999914e+107 | "#NUM!" |

## Acceptance criteria
- [ ] A probe case with oracle values for MINVERSE (INDEX into each cell, or a spill) of {1,2,3;4,5,6;7,8,9},
      {1,2;2,4}, {1E-20}, {1E-17,0;0,1}, {1,1;1,1.0000000000000002} and {1E-300}.
- [ ] The same probe also covers MDETERM of a matrix whose pivot product overflows, e.g. {1E+200,0;0,1E+200}.
      The engine gives `#NUM!` there (T-1751), but that is unprobed.
- [ ] MDETERM of tiny-pivot matrices: {1E-20}, {1E-300}, {1E-17,0;0,1}, {1E-200,0;0,1E-200} (underflow). The engine
      keeps any nonzero pivot (T-1751), which is evidenced only down to a 1E-16 residual.
- [ ] arrays.md records the rule. A follow-up implementation task is filed if the engine differs, or the fix
      goes in this PR when it is a one-line threshold change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MDETERM (T-1751).
