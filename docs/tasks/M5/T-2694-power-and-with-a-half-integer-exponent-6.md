---
id: T-2694
title: "POWER and ^ with a half-integer exponent: 6 misses in over 7,000 probes-numeric cells, one visibly off (t2513-r3-tdensity-hold I563)"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
POWER (and the `^` operator, which shares `arithmetic::power`) gives Excel's value for the six half-integer-exponent cells
below, and at least Excel's 15 significant digits for the one that is visibly off, without losing any passing POWER cell.

## Context
Gap triage 2026-10-25 (`make eval`). The probes-numeric cases of the T-2513 and T-2586 density studies use POWER as a
building block with half-integer exponents from -50.5 to 50.5: over 7,000 such cells, and all but six match Excel to the
bit. The six misses (excel-graph), with a high-precision reference computed from the double base the formula produces
(Python `decimal`, 50 digits):
- probes-numeric/t2513-r3-tdensity-hold Calc!I563 `=POWER(12/(12+A563*A563),6.5)`: Excel 9.174378504956638E-06, ours
  9.174378504956655E-06 (visibly off: 15th digit). Reference 9.17437850495664684E-06: Excel is 4.4 ulp low, ours 3.9 ulp high.
- t2513-r3-tdensity-hold Calc!I475, I476, I477 `=POWER(12/(12+A475*A475),6.5)`: Excel 0.4613031603697908, ours
  0.46130316036979085 (1 ulp; reference 0.46130316036979083).
- t2513-r3-tdensity-a2 Calc!D653 `=POWER(1+A653*A653/10,-5.5)`: Excel 0.32752527329759007, ours 0.3275252732975901 (1 ulp).
- t2586-r4-f-small-df-a Calc!D4143 `=POWER(1+2/(5*A4143),-3.5)`: Excel 0.03636959589411698, ours 0.03636959589411699
  (Excel 2.2 ulp low).
Excel is below the reference in all six. Libm `pow`, `x^n·sqrt(x)` and `sqrt(x)^(2y)` each match some of them but not
all, so none of them is the fix. Model: `power` / `non_integer_power` in crates/rustsheet-engine/src/arithmetic.rs computes
`excel_exp(extended_product(y, ln x))`, which matches all 180 cells of probes-numeric/power-non-integer-sweep
(docs/excel-semantics/numbers.md "Non-integer exponents: exp(y·ln x)"). The remaining suspects are the `x.ln()` step (libm
`ln`, while LN itself has an x87 FYL2X model, T-2651) and the operands themselves (`12+A*A`, `/`, `A*A/10` through the
engine's arithmetic rules). Check first that the engine's base equals the double the reference used. A one-ulp change in
the base moves a 6.5th power by about 6.5 ulp, so the base alone can explain I563.
The case inputs are exact binary fractions (`A563` is `=(64694343*2^-23+57992555*2^-49)`), so the cells can be replayed
offline. If the six cells cannot tell candidate models apart, do not fit blind: file a research sweep (half-integer
exponents 3.5 to 10.5 over bases in (0,1) and (1,3), with `LN` and `EXP` twins of each cell) and block on it.

## Acceptance criteria
- [x] Visible parity: probes-numeric/t2513-r3-tdensity-hold I563 is within half a unit in the 15th significant digit of
      Excel's value.
- [x] The six cells above match Excel to the bit, or the ones that do not are moved to a `defer: phase-1b` follow-up with the
      evidence gathered; no POWER, `^`, MIRR or other cell that passes today fails (`make eval`: no new line in any baseline,
      probes-numeric power-* cases still pass in full).
- [x] numbers.md "Non-integer exponents" records what was learned (the rule or the open question).
- [x] `make check` green; reviewers approve.

## Out of scope
- Integer exponents (square-and-multiply, already matched) and the T-2513/T-2586 density formulas built on POWER.

## Visible-parity stage
Decision 0083: one cell (I563) is visibly off, and Excel is not correctly rounded there (4.4 ulp below the reference), so
this is the `visible` stage (decision 0114): reproduce Excel's algorithm, time-boxed (decision 0056). The first acceptance
criterion is the target; the bit-level residue may move to a deferred follow-up.

## Outcome
POWER and `^` take `ln x` from LN's own x87 FYL2X kernel (`excel_ln`) instead of libm `ln`; the rest of the model
(`excel_exp` of the twice-rounded product, one over the positive power) is unchanged. Offline replay of the six cells
(bases through the engine's `*`, `+`, `/`) under five candidates (libm or Excel `ln`, double or 64-bit `ln`, double or
64-bit product) left exactly one that matches all six to the bit: Excel's `ln` stored as a double. The base was the double
the formula gives; it was not the source. MIRR's desktop root keeps libm `ln` and `exp` (`power_with_libm_exp`).
Eval: probes-numeric passes 297,903 -> 297,909 of 611,774 cells (the six, I563 among them; no cell lost); gated suites:
no change, no regressions. numbers.md "Non-integer exponents" records the rule; unit test
`non_integer_powers_take_excel_ln_of_the_base`.
