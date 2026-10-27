---
id: T-2923
title: "SPEC-0024 amendment: LAMBDA calls cost far more than their body's nodes; charge a call or bound nesting"
milestone: M7
lane: engine
status: done
priority: P2
size: S
tier: opus
spec: docs/specs/SPEC-0024-formula-work-budget.md
depends_on: []
---

## Goal
SPEC-0024 says how a LAMBDA call is charged to the formula work budget so that its section 4 upper constraint
(each kind's worst case reaches the limit within 10 s, release build) holds for nested LAMBDA helpers. T-2803 can then
finish its calibration.

## Context
SPEC-0024 section 1 says "The LAMBDA call itself needs no separate charge, because its body's nodes pay for it."
T-2803's resumed run (2026-10-27, release build, its branch with origin/main merged in, an idle 4-core container) shows that
this is false. Committed weights: `NODE_UNITS` 2, `CELL_UNITS` 2, `TEXT_BYTES_PER_UNIT` 4, `VISIT_UNITS` 1. Time and units:

| formula | units | time |
|---|---|---|
| `SUM(MAKEARRAY(1024,1024,LAMBDA(r,c,r)))` | 4,194,314 | 0.29 s (about 70 ns a unit, 275 ns a call) |
| `SUM(MAKEARRAY(1024,1024,LAMBDA(r,c,MAP(1,LAMBDA(z,z)))))` | 12,582,922 | 1.84 s (146 ns a unit) |
| `SUM(MAKEARRAY(1024,1024,LAMBDA(r,c,MAP(SEQUENCE(1),LAMBDA(z,r+c+z)))))` (SPEC's node case) | 25,165,834 | 4.58 s (182 ns a unit), completes |
| `SUM(MAP(SEQUENCE(1000),LAMBDA(x,SUM(MAKEARRAY(1024,1024,LAMBDA(r,c,r))))))` | to the limit | 17.9 s |
| `SUM(MAP(SEQUENCE(1000),LAMBDA(x,SUM(MAKEARRAY(1024,1024,LAMBDA(r,c,MAP(SEQUENCE(1),LAMBDA(z,r+c+z))))))))` | to the limit | **124 s** (460 ns a unit) |
| the three Problem formulas (REDUCE, nested MAP, MAKEARRAY of `SUM(SEQUENCE(1000))`) | to the limit | 10.5, 10.6, 11.1 s |

The last nested row costs about 2.7 times as much per cell as the same body under one helper fewer (11.8 vs 4.4 us). So the
cost of a call seems to grow with the nesting depth or with the arrays the outer helpers hold, not only with the body's nodes.
No node weight bounds that, so T-2803 stopped at section 4's rule ("no weights meet both constraints").

Work: find where the per-call time goes (`call_prepared`, `lambda_stack_base`, the binding stack and captured cells, nested
1x1 array results of MAP inside MAKEARRAY; profile the 124 s formula). Then amend SPEC-0024 sections 1 and 4 with one of these:
a per-call charge (`CALL_UNITS` in `call_prepared`, fitted by time), a charge that scales with what the call costs, or a perf fix
that makes the call cost what its nodes are charged (filed as its own task). Also say whether the per-unit target for nodes and
built cells (now about 40 ns a unit at weight 2) should move to weight 3, so the Problem formulas stop within 10 s with margin.
The lower constraint must still hold: the heaviest eval formula outside the visit caps spends 8,388,620 units
(ironcalc-calc/LOOKUP_AND_REFERENCE/ROW_COLUM.xlsx, a whole-column ROW array). The heaviest with them spends 118,754,196
(gen-m9/3e98b26cf6713e77 Calc!A204, a lifted CHISQ.INV.RT over a PIVOTBY: 118,751,450 visit units).

## Acceptance criteria
- [x] The cause of the nested-call cost is named, with a profile or counters.
- [x] SPEC-0024 sections 1 and 4 are amended with the chosen charge, or with a perf task that removes the cost. A worst case of
      nested LAMBDA helpers (at least the 124 s formula above, and a deeper one) is added to section 4's timed list.
- [x] T-2803's "Blocked because" names this amendment as its blocker; it is unblocked when this merges.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the budget (T-2803).
- The visit caps' rule (T-2813, done).

## Outcome
The time goes to making LAMBDA values, not to calling them, and not to nesting depth. The run timed 16 variants on main
(release build, a scratch test, not committed):
- A call whose body makes no LAMBDA costs about 275 ns (`MAKEARRAY(1024,1024,LAMBDA(r,c,r))`: 0.26 s). That is the same at
  depth 3 (`MAP` of `MAP` of 32 by 32 MAKEARRAYs: 0.23 s for 1M calls).
- Making a LAMBDA costs about 1 us more (`MAP(1,LAMBDA(z,z))` 1.40 s, `LAMBDA(z,z)(1)` 1.31 s, `SEQUENCE(1)` 0.28 s per 1M
  cells). `LambdaDefKind` clones the body into `self.lambdas`. When the new LAMBDA reads the call's parameters, the caller's
  body is also patched afresh on every call (`call_prepared`).
- The cost per cell grows with how many LAMBDAs are stored, not with depth. `self.lambdas` keeps them until the evaluation
  ends: 4.49 s for 1M made, 21.8 s for 4M in one formula, 4.48 s for 1M made at depth 3 over small arrays.

SPEC-0024 now charges LAMBDA values made: `LAMBDA_UNITS` per LAMBDA stored, plus the nodes of each body clone (section 1).
Section 4 times three nested worst cases and records that the Problem formulas took 10.5-11.1 s at weight 2, so step 1
re-fits `NODE_UNITS` and `CELL_UNITS` with a margin. Filed T-2925: drop a formula's LAMBDAs after its root cell. Eval delta: none
(documentation only).
