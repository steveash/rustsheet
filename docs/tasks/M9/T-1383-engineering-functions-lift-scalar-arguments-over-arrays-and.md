---
id: T-1383
title: "Engineering functions lift scalar arguments over arrays, and BIT* over ranges (probes/eng-argument-modes)"
milestone: M9
lane: functions-engineering
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
Every scalar argument of the M9 engineering functions lifts elementwise over an array literal, and
BITAND, BITOR, BITXOR, BITLSHIFT and BITRSHIFT also lift over a multi-cell range, as Excel does. The
other engineering functions stay `#VALUE!` over a multi-cell range.

## Context
SPEC-0011 rollout step 2 (docs/specs/SPEC-0011-scalar-argument-lifting.md). Every engineering function is `LiftArrays` in every argument, except BIT* (`Lift`) and IMSUM/IMPRODUCT (`WholeByCaller`, T-1384). The later arguments are now probed (probes/scalar-lift-shapes H7-H10): `BITAND(13,D1:D60)` lifts to 60 rows, while BESSELJ's n, DEC2BIN's places and DELTA over a range are `#VALUE!`, even on the range's own rows. Those three arguments are now evidenced; the others still need the probe below. BESSELI's intersection goes away because the driver never hands a `LiftArrays` kernel a multi-cell range. Before entering a later scalar argument as `LiftArrays`, probe it over a multi-cell range on the range's own rows (a probe case in this task, excel-probe skill), unless SPEC-0011 or T-1777 already cites a cell for it. Only the first scalar argument of each function and the later ones listed in docs/excel-semantics/arrays.md "Scalar arguments lift ..." are evidenced. The table describes a function whole, so a function with an unprobed argument is entered only once that argument is probed. Until then it keeps today's code.

Evidence: probes/eng-argument-modes (excel-graph, 2026-10-11, T-1085), docs/excel-semantics/engineering.md
"Engineering function argument modes". Known failures:
- columns B:D, `ROWS(F(..{v;v}..))` should be 2 for every argument of every function except IMSUM and
  IMPRODUCT. Today we give `#N/IMPL!` (84 cells) or `#ERROR!`.
- E8:E12, `ROWS(BITAND(K15:K16,25))` and the other BIT* functions over a two-cell range should be 2.
  Today we give `#VALUE!`.
- E1, `ROWS(BESSELI(K1:K2,1))` should be `#VALUE!`. Today we give 1 because the range is implicitly
  intersected; the other Bessel functions already give `#VALUE!`.

Reuse the lifting helper that the financial (T-1372) and statistical (T-1290) lifting work uses, if one
exists by then (`rg -n "lift" crates/rustsheet-engine/src/functions`). Do not add a per-function copy.

Only the first argument was probed over a range. Probe the later arguments (BIT* `shift_amount` and
`number2`, `places`, Bessel `n`) before range lifting is extended to them.

## Acceptance criteria
- [ ] probes/eng-argument-modes columns B:E leave eval/baseline/probes.txt, except the IMSUM and IMPRODUCT
      cells (T-1384), with no new F lines. probes/scalar-lift-shapes H7-H10 pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMSUM and IMPRODUCT (T-1384). Last-digit precision of IM* (T-0907).
