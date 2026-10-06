---
id: T-0196
title: "3-D references in AVERAGEA, MAXA, MINA, PRODUCT, CONCAT, LARGE and RANK"
milestone: M1
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-0122]
---

## Goal
AVERAGEA, MAXA, MINA, PRODUCT, CONCAT, LARGE and RANK accept 3-D references (`Data1:Data3!A1:A3`) like SUM does.

## Context
T-0122 made 3-D references work through `evaluate_aggregate_arg` and the shared range walk
`for_each_cell_or_error_in_range` (crates/rustsheet-engine/src/functions/util.rs), which walks a
cross-sheet `Range` sheet by sheet. AVERAGEA, MAXA, MINA, PRODUCT and CONCAT evaluate their arguments with
`evaluate_node_in_context`, so a 3-D reference reaches them as `#REF!`.
Failing cells: probes/3d-ref-aggregates Calc!C6 PRODUCT 1000, C8 AVERAGEA 15.714285714285714,
C9 MAXA 100, C10 MINA -4; probes/3d-ref-values Calc!C8 CONCAT "110100"; probes/3d-ref-functions
Calc!C11 LARGE 100 and C14 RANK 2 (both `#N/IMPL!` today: the `_ =>` arms in
count_and_average.rs (LARGE) and rank_eq_avg.rs (RANK) turn the `#REF!` into NIMPL).
Rule: docs/excel-semantics/references.md "3-D sheet-range references in aggregates".
Moving the argument evaluation onto `evaluate_aggregate_arg` also changes how IF/INDEX arguments
arrive (as references); check those functions' existing cases still pass.

## Acceptance criteria
- [ ] The seven cells above pass; no new F lines in eval/baseline.
- [ ] No per-function sheet loop: the 3-D walk stays in `for_each_cell_or_error_in_range`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions (STDEVA, VARA, ...): probe first, file separately.
