---
id: T-2925
title: "Drop the LAMBDAs a formula made once it is evaluated, so stored LAMBDAs do not pile up across a recalculation"
milestone: M7
lane: engine
status: done
priority: P2
size: S
tier: opus
spec: docs/specs/SPEC-0024-formula-work-budget.md
depends_on: [T-2803]
---

## Goal
The LAMBDAs one formula makes are released when its root cell has been evaluated. A recalculation of many formulas that make
LAMBDAs then holds at most one formula's worth at a time, and a formula's LAMBDA calls do not slow down with what earlier
formulas made.

## Context
T-2923 (SPEC-0024 amendment, "LAMBDA values made") found that `Model::store_lambda` keeps every LAMBDA value made in
`self.lambdas`, with a clone of its body (node_frames.rs `LambdaDefKind`). `clear_lambdas` empties that map only between
evaluations (evaluation.rs, data_tables.rs). `evaluate_root_cell` already drops `lambda_captures` per root cell, because no
LAMBDA outlives the formula that made it (a cell holding one is `#CALC!`), but it keeps `lambdas`. Measured on main (release,
2026-10-27): `SUM(MAKEARRAY(1024,1024,LAMBDA(r,c,MAP(SEQUENCE(1),LAMBDA(z,r+c+z)))))` stores 1,048,576 LAMBDAs in 4.5 s
(4.3 us a cell). Four times that inside one formula takes 21.8 s (5.2 us a cell), and about 11M takes 11.8 us a cell. So the
cost grows with how many LAMBDAs are stored, and memory grows with them, a body clone each. SPEC-0024's work budget bounds the
LAMBDAs one formula makes (its "LAMBDA values made" charge, T-2803). Nothing bounds those of a whole recalculation.

Work: in `evaluate_root_cell`, drop the root formula's entries of `lambdas`, `eta_lambdas` and `named_lambdas`, as
`lambda_captures` is dropped. Check first what still reads a LAMBDA id after its root cell: a spilled LAMBDA is `#CALC!`, a
defined-name LAMBDA's id is cached per evaluation (`NamedLambdas`), and a `PreparedLambda`'s `lambda_mark` compares ids, so ids
must keep growing (do not reset `last_lambda_id` per root cell). Keep the drop proportional to what the formula made (ids from
the root's start mark), not to the whole map.

## Acceptance criteria
- [x] After a root cell is evaluated, `lambdas` holds none of the LAMBDAs its formula made (an engine test over two cells,
      counting the map's entries, not timing).
- [x] Every LAMBDA test and the probes/lambda-* eval cases still pass; no baseline gains a line.
- [x] The Outcome records the time of 16 cells of `SUM(MAKEARRAY(1024,1024,LAMBDA(r,c,MAP(SEQUENCE(1),LAMBDA(z,r+c+z)))))`
      before and after (release build).
- [x] `make check` green; reviewers approve.

## Out of scope
- The per-formula charge for LAMBDAs made (T-2803, SPEC-0024 as amended by T-2923).
- The other retained memory of one formula (T-2806).

## Outcome
`Model::evaluate_root_cell` now drops the LAMBDAs its formula made (`lambdas`, `eta_lambdas`, and the defined-name
entries of `named_lambdas` from the root's start id on), in time proportional to what it made; `last_lambda_id` keeps
growing, and a table left large by a formula that made many is given back. Engine test
`a_root_cells_lambdas_are_dropped_once_it_is_evaluated` (fails without the drop). Timing (release build, this container,
2026-10-28; 16 cells of `SUM(MAKEARRAY(1024,1024,LAMBDA(r,c,MAP(SEQUENCE(1),LAMBDA(z,r+c+z)))))`, each of which reaches the
SPEC-0024 work budget and is `#VALUE!` since T-2803's LAMBDA charge): 36.8 s before, 32.7 s after (about 11% faster; the
work budget, not the map, now bounds each cell). Eval delta: none expected (no value changes).
