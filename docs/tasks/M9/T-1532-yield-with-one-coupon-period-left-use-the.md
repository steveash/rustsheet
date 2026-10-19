---
id: T-1532
title: "YIELD with one coupon period left: use the closed form (bit-exact on probes/solver-iteration-traces Y!A, Y!B)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-1525]
---

## Goal
YIELD returns Excel's bits when at most one coupon period is left before maturity, on every basis.

## Context
T-1525 found that Excel does not iterate in this case. It evaluates the documented closed form
`((red/100 + rate/f) - (pr/100 + A/E*rate/f)) / (pr/100 + (A/E)*(rate/f)) * (f*E/DSR)` in that
order of operations, with these day counts:
- DSR is the settlement-to-maturity day count of the basis: DAYS360 US on basis 0 (180 for
  2024-03-01 to 2024-08-31, while E - A is 179), DAYS360 European on basis 4, actual days on bases 1-3.
- E is 360/f on bases 0 and 4 but the actual coupon-period length on bases 1, 2 and 3 (182 and 366
  days for the two probe bonds, not 360/f or 365/f).

Evidence: `uv run rustsheet-fitlab yield-one-period` (from `tools/`) is bit-exact on 60 of 60 cells:
probes/solver-iteration-traces Y!A1:A10, Y!B1:B10 (basis 0) and probes/bond-one-period-bases sheet Y
(bases 0 to 4, two bonds, four prices each). Using PRICE's E - A for DSR gives 30/60; E = 360/f and
365/f on bases 2 and 3 gives 44/60. See `docs/excel-semantics/financial.md`, "YIELD with one coupon
period left is closed form".

The engine today gets 0/60: `fn_yield` (`crates/rustsheet-engine/src/functions/financial/financial_bonds.rs`)
solves `compute_price` numerically, and `compute_price`'s one-period branch uses DSC/E = (E - A)/E.
Where E - A equals Excel's DSR and E (basis 0 on the semiannual bond, bases 1 and 4 on both) the result
is the right root a few to about 15 ulp off; elsewhere it is the wrong formula (basis 0 annual, Y!B: all 10 cells about 0.56% off,
ratio 180/179; Y!B1 Excel 0.2791614406020427, engine 0.2807210017227244; bases 2 and 3 more).
`compute_price` is shared with PRICE, and PRICE is right as it is: probes/bond-one-period-bases sheet P
(40 cells, the same bonds on bases 0 to 4) shows PRICE uses DSR = E - A with the nominal E, and the
engine passes all 40. So the fix is a separate one-period YIELD formula with its own DSR and E; do not
change `compute_price`.

## Acceptance criteria
- [x] YIELD with one coupon period or less left uses the closed form above, in Excel's order of operations, on every basis.
- [x] probes/solver-iteration-traces Y!A1:A10, Y!B1:B10 and probes/bond-one-period-bases Y!A1:J4 pass; PRICE (sheet P) still passes; no other cell regresses (`make eval-baseline`).
- [x] `make check` green; reviewers approve.

## Out of scope
- YIELD with more than one coupon period (its solver is unfitted: T-1533, T-1374).

## Outcome
New `yield_one_period` in `financial_bonds/yield_closed.rs`: when one coupon is left YIELD evaluates the closed form with DSR = basis settlement-to-maturity count and E = actual period on bases 1-3. `compute_price` untouched. Eval: probes 2130 -> 2070 known failures (60 fixed, 0 regressions).
