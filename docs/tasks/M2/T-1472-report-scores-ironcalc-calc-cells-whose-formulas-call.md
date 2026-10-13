---
id: T-1472
title: "Report scores ironcalc-calc cells whose formulas call only M0-M2 functions (M2 exit criterion)"
milestone: M2
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`make eval` reports the M2 exit criterion "ironcalc-calc cells using only M1/M2 functions at 100%"
(docs/milestones.md) as a number, with the failing cells listed. It is no longer computed by hand.

## Context
The 2026-10-13 planner computed it with a script. Take the ironcalc-calc cells in report.json whose formula is
known. Keep those where every function the formula calls (`report::functions_in`) has a catalog milestone of
M0, M1 or M2 (eval/catalog via `Catalog`). Formulas that call no function count too. Result: 9 failing cells,
all in `ironcalc-calc/FINANCIAL/TBILLs.xlsx` (L5, L6, L7, L9, L10, L16, L20, L24, L26: `=(360-365*C5/D5)/C5`,
whose D input is a TBILLEQ result; T-1475). Spill children have no formula of their own in the snapshot
(46 failing cells: IF_ARRAY, POWER, SEQUENCE, LINEST/TREND). They are left out and counted on a separate line,
so the gap stays visible.

## Acceptance criteria
- [x] report.md gets a "Milestone function scope" table for ironcalc-calc: cells scored, bit-exact, failing (with
      case/cell ids), and spill children left out. report.json carries the same data.
- [x] The function-to-milestone lookup reuses `Catalog` and `functions_in`, with no second parser of the catalog.
- [x] A unit test covers a function-free formula (in scope), an M2-only formula (in scope) and a formula calling an M9 function (out).
- [x] On today's tree it reports the 9 TBILLs.xlsx cells above as failing.
- [x] `make check` green; reviewers approve.

## Out of scope
- Following precedents transitively (decide that in the milestone text if wanted; note it in the Outcome).
- Other suites.

## Outcome
`make eval` now prints a "Milestone function scope (ironcalc-calc)" section (report.md, and
`function_scope` in report.json): 3977 scored formula cells whose functions are all M0–M2 (or none),
3968 bit-exact, 9 failing (the TBILLs.xlsx L5, L6, L7, L9, L10, L16, L20, L24, L26 cells), plus 1378
formula-less cells (spill children) left out, 46 of them failing. Implemented in
`crates/rustsheet-eval/src/function_scope.rs` on `Catalog` and `functions_in`; the milestone parser
is the one in `probe_milestones`. A function the catalog does not list counts as in scope, as in
`Catalog::in_scope`. Precedents are not followed transitively: the TBILLs cells fail because of a
TBILLEQ precedent (T-1475), not their own formula. Eval delta: none (reporting only).
