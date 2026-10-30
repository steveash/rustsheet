---
id: T-3204
title: "rustsheet-gen --policy numeric-fp: extend to M5 rounding chains (ROUNDUP, ROUNDDOWN, TRUNC, INT, MOD) and 15-digit boundaries"
milestone: M5
lane: harness
status: todo
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
- [ ] `rustsheet-gen --policy numeric-fp --milestone M5` (or an equivalent flag) writes deterministic gen-m5 cases
      with `hypotheses` per cell: TRUNC, INT, MOD, ROUNDUP and ROUNDDOWN as pickers and as wrappers of `X-b`, and
      operands at 15-digit cut boundaries (X whose 15-digit cut is or is not b), in both operand orders; only
      catalog M0-M5 functions appear.
- [ ] tools tests cover determinism, spec2xlsx parsing and the function scope.
- [ ] Oracle-filled on excel-graph and baselined with no regressions; every failing cell has an open owner
      (`make exit-owners MILESTONE=M5` reports 0).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for any failures (owner tasks).
- The other §5.5 policy rows (`coercion`, `error_prop`, `lookup`, ...).
