---
id: T-3204
title: "rustsheet-gen --policy numeric-fp: extend to M5 rounding chains (ROUNDUP, ROUNDDOWN, TRUNC, INT, MOD) and 15-digit boundaries"
milestone: M5
lane: harness
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
gen-m5 gets seeded, oracle-filled cells in which the M5 rounding functions (TRUNC, INT, MOD) and the M2
ROUNDUP/ROUNDDOWN supply, wrap or consume a near-zero cancellation, plus operands at the 15-significant-digit
boundary, so the M5 exit suite covers the snap and the ROUND family's 15-digit cut together.

## Context
- T-3139 added `rustsheet-gen --policy numeric-fp` (tools/src/rustsheet_tools/gen/numeric_fp.py) for M2 only:
  root pickers, M2 wrappers and summing functions over X = base*(1±k*2^-52) against b = base (5,200 gen-m2 cells,
  all matching). Its §5.5 note in docs/architecture/synthetic-generator.md says ROUND/TRUNC/MOD/INT chains past M2
  and the 15-digit boundaries are not covered.
- The T-3139 data show ROUND cuts its operand to 15 digits before rounding (`ROUND(X,20)-ROUND(b,20)` snaps far
  more often than `X-b`); TRUNC, INT, MOD and ROUNDUP/ROUNDDOWN at the same operands are unprobed in a generated
  suite. Rules: docs/excel-semantics/numbers.md "Near-zero snapping", "ROUND family rounds in two decimal stages",
  the MOD entries (T-2654, T-2714).
- Reuse `numeric_fp`'s pair layout and `numeric_rules`' `BASES`/`fmt_k`; add a milestone parameter or a sibling
  position set rather than copying the module.

## Acceptance criteria
- [x] `rustsheet-gen --policy numeric-fp --milestone M5` (or an equivalent flag) writes deterministic gen-m5 cases
      with `hypotheses` per cell: TRUNC, INT, MOD, ROUNDUP and ROUNDDOWN as pickers and as wrappers of `X-b`, and
      operands at 15-digit cut boundaries (X whose 15-digit cut is or is not b), in both operand orders; only
      catalog M0-M5 functions appear.
- [x] tools tests cover determinism, spec2xlsx parsing and the function scope.
- [x] Oracle-filled on excel-graph and baselined with no regressions; every failing cell has an open owner
      (`make exit-owners MILESTONE=M5` reports 0).
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for any failures (owner tasks).
- The other §5.5 policy rows (`coercion`, `error_prop`, `lookup`, ...).

## Outcome
`rustsheet-gen --policy numeric-fp --milestone M5` (numeric_fp.py, sharing the M2 pair layout, `_inputs` and
`_case`; the M2 output is byte-identical) writes three gen-m5 cases: `root` (TRUNC with and without 20 digits, INT,
`MOD(..,2^21)`, ROUNDUP and ROUNDDOWN at 20 as pickers, both orders, 1,200 cells), `wrapper` (TRUNC both forms, INT,
`MOD(..,1)`, ROUNDUP at 20 and 0, ROUNDDOWN at 20 around `X-b`, both orders, 1,400 cells) and `digits15` (k one step
either side of each base's rounded and truncated 15-digit cut, every picker and wrapper, tagged
`round15=`/`trunc15=`, 1,652 cells). Oracle-filled on excel-graph 2026-10-30. Eval delta: gen-m5 +4,252 checked
cells, all passing (0 known-failure lines added; the suite's one F line is pre-existing); `make exit-owners
MILESTONE=M5` reports 0. Findings in numbers.md "Near-zero snapping" (T-3204 paragraph): rounding wrappers keep and
then round the residue (at X = 1+2^-52, `INT(b-X)` = -1, `MOD(b-X,1)` = 1-2^-52), a root over rounded operands snaps by the
left-addend bound on the cut values, and the 20-digit TRUNC/ROUNDUP/ROUNDDOWN follow the rounded 15-digit cut, not
the truncated one.
