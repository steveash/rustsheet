---
id: T-2207
title: "PRICE and ODDFPRICE to 15 significant digits: discount each coupon with Excel's ^ kernel, off powf"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
PRICE and ODDFPRICE are within half a unit of the 15th significant digit of Excel's value on every
cell in eval/ (bit-exact where the evidence below says the model gives Excel's bits), because each
coupon and the redemption are discounted with the engine's `arithmetic::power` (Excel's `^`)
instead of libm `powf`. YIELD and ODDFYIELD, whose solvers call the same price functions, get no
new visibly-off cell.

## Context
Gap triage 2026-10-20, split from T-1374 (which keeps the bit-level residue of DURATION, MDURATION and
CUMIPMT/CUMPRINC and any PRICE/ODDFPRICE cell this task leaves).

Code (main 4ff52373), `crates/rustsheet-engine/src/functions/financial/financial_bonds.rs`:
- `CouponSchedule::price` (PRICE, and YIELD's solver since T-1842): `redemption / v.powf(dsc_over_e + n - 1.0)`
  and `coupon / v.powf(t)` per coupon.
- the odd-first-period price (ODDFPRICE, and ODDFYIELD's solver): `x.powf(self.front_exponent)`,
  `x.powf(self.lead_exponent)`, `x.powf(index + offset + y)`.
- DURATION/MDURATION in the same file already discount with `power(v, t)` (T-1975, fitted), and PV/FV use
  `power(extended_sum(1.0, rate), nper)` (T-0901).

Evidence that the kernel is the cause (triage experiment, fit lab model
`tools/src/rustsheet_tools/fitlab/kernels/power.py` `excel_model`, the model `arithmetic::power`
implements): with integer exponents (settlement on a coupon date, so `dsc_over_e` = 1),
`redemption / v^n + sum(coupon / v^k)` with Excel's `^` gives Excel's bits on all 8 such cells of
ironcalc-calc FINANCIAL/PRICE_YIELD.xlsx (PRICE_YIELD!H2, H3, H4, H5, H8, H9, H10, H11, e.g. H2
108.22689722310892) and on probes/fin-coupon-walk-bounds-c C10 `PRICE(1,43831,0.05,0.06,100,2)`
83.34716628375435 (240 coupons); libm `pow` gives the engine's current values on all 9 (H2
108.22689722310889, C10 83.34716628375423). Fractional exponents (`dsc_over_e` < 1, ODDFPRICE's
offsets) go through `power`'s exp(y ln x) path and are not checked yet.

Failing cells (`make eval` on a15f80ae, 2026-10-20):
- Visibly off (decision 0083): PRICE 6, probes/fin-coupon-walk-bounds-c C10, C11, G10, G11, J10, J11
  (83.37652817137169 vs ours ...145 at G10); ODDFPRICE 14, ironcalc-calc PRICE_YIELD.xlsx
  ODDFPRICE_ODDFYIELD!J31, probes/fin-argument-modes H189, probes/fin-bond-negative-basis E9, G9,
  probes/fin-remaining-later-argument-ranges I105-I119 (odd rows), probes/fin-yield-odd-lift A3, I7
  (`ODDFPRICE(39448,43831,39083,39630,0.08,0.09,100,2,0)` Excel 92.4077632093273, ours 92.40776320932721).
- Ulp-level: PRICE_YIELD.xlsx PRICE_YIELD!H2-H6, H8-H11, H28; ODDFPRICE_ODDFYIELD!J5, J8, J9, J11, J23,
  J25, J27, J29, J30, J32, J33, J44, J45; ODDFPRICE_SHORT!J9, J10, J13; probes/fin-argument-modes H251,
  fin-bond-negative-basis E2, G2, fin-remaining-later-argument-ranges I165-I171 (odd rows).

Solver interplay: YIELD (`CouponSchedule::price` inside its Newton loop) and ODDFYIELD stop short of the
root in Excel (financial.md "YIELD stops short of PRICE's root"; T-1592, T-2187, T-2007), so changing the
price function moves every YIELD and ODDFYIELD iterate. The YIELD lab prices with libm: `price()` in
tools/src/rustsheet_tools/fitlab/kernels/solvers.py discounts with `rpow` (`math.pow`), so the
2/90 YIELD fit (T-1525, T-1592) was scored on libm prices. Add a `^`-priced variant (the `excel_model`
of kernels/power.py), re-score `uv run rustsheet-fitlab yield` with it and record both counts in
financial.md and in T-1592's and T-2007's Context (a better YIELD count is new evidence for those fits). A YIELD/ODDFYIELD cell may move, within decision 0083's allowance (each regressed cell stays
within 4 ulp of Excel and none becomes visibly off); if that does not hold, keep the solver on a
separate libm-priced function and say why in financial.md.

## Acceptance criteria
- [ ] Every PRICE and ODDFPRICE cell listed above is within half a unit of Excel's 15th significant
      digit; the 9 integer-exponent PRICE cells (PRICE_YIELD!H2-H5, H8-H11, fin-coupon-walk-bounds-c C10)
      pass bit-exact. Their known-failure lines leave eval/baseline/ironcalc-calc.txt and
      eval/baseline/probes.txt.
- [ ] No new line in any baseline, or each new YIELD/ODDFYIELD line is within decision 0083's
      allowance and listed in the Outcome with its ulp distance.
- [ ] A unit test pins `PRICE(DATE(2020,3,1),DATE(2030,3,1),0.06,0.07,130,1,0)` = 108.22689722310892
      and fails on main.
- [ ] docs/excel-semantics/financial.md records that PRICE and ODDFPRICE discount with Excel's `^`
      (with the cells above as evidence), and how the fractional-exponent cells came out.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit parity of the YIELD and ODDFYIELD stopping points (T-1592, T-2187, T-2007).
- DURATION, MDURATION, CUMIPMT, CUMPRINC (T-1374, T-1890); ODDLPRICE/ODDLYIELD unless they share the
  changed function (then include them and list their cells).

## Visible-parity stage
Decision 0083: wrong bond prices are visible in the 15th digit. The first acceptance criterion is the
visible-parity stage; bit-level misses left on fractional-exponent cells go back to T-1374.
