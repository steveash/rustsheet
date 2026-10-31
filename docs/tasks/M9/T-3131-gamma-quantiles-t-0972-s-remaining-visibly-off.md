---
id: T-3131
title: "Gamma quantiles: T-0972's remaining visibly-off cells and its statrs search start"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: S
tier: opus
campaign: visible
depends_on: [T-0972]
---

## Goal
The cells T-0972 owned that are still visibly off after it merged reach visible parity, and the gamma quantile no
longer starts from the ported statrs search.

## Context
T-0972 (#1767) refines GAMMA.INV, CHISQ.INV, CHISQ.INV.RT, GAMMAINV and CHIINV to the crossing of the smaller tail
(`functions/statistical/incomplete_gamma/mod.rs` `gamma_quantile`). It closed at decision 0083's visible-parity stage
with these owned cells still past half a unit of the 15th digit (local `make eval` on the T-0972 branch, 2026-10-29;
the two cells T-0972 made newly visibly off belong to T-3239 and T-3238, after T-3016):
- gen-m9/4250f880ab640c49 Calc!A371 `CHISQ.INV.RT(((-0.0001)+1),3)`: Excel 0.005214832329030269, ours 0.005214832329027594
- gen-m9/73f053591a54bc43 Calc!A108 `NORMSDIST(MOD(SUMIF(Data!A10:D12,"",Data!B4:E12),CHISQ.INV.RT(0.975,5)))`: Excel
  0.6004864351048662, ours 0.6004864351049074
- gen-m9/a5828d3925575713 Calc!A326 `CHISQ.INV(0.975,3)`: Excel 9.348403604496152, ours 9.348403604496145
- gen-m9/d8266bc15533702a Calc!A29 `CHISQ.INV(T.DIST((7-1),6,("x~*"="x~*")),5)`: Excel 22.187751453507733, ours
  22.18775145350798 (T.DIST's share is T-0969's)
- probes-numeric/t1049-chisqinvrt Calc!A19 `CHISQ.INV.RT(0.999,5)`: Excel 0.210212602629218, ours 0.21021260262921926
- probes-numeric/t1049-chisqinvrt Calc!A20 `CHISQ.INV.RT(0.999,20)`: Excel 5.921040745487456, ours 5.92104074548752
- probes-numeric/t1049-chisqinv Calc!A27 `CHISQ.INV(0.5,1E10)` and t1049-chisqinvrt Calc!A27 `CHISQ.INV.RT(0.5,1E10)`:
  Excel 9999999999.333334, ours `#NUM!` (shape above `REFINE_SHAPE_MAX`, statrs' search alone; check T-2982 first)
  (2026-10-30: T-3070 removed `REFINE_SHAPE_MAX` and gives `CHISQ.INV(0.5,1E10)` and `CHISQ.INV.RT(0.5,1E10)` Excel's
  value to 15 digits, 2 ulps; these two t1049 A27 cells are no longer this task's, T-3272 owns their last digits.)

Near p = 0.999 Excel's double cdf is flat and Excel is far from the true quantile (`docs/excel-semantics/statistics.md`,
the T-0972 bullet), so those cells need Excel's iteration, not more accuracy.

T-0972's criterion 3 is also open: `inverse_cdf_within` (the ported statrs search) is still the refinement's start.

## Acceptance criteria
- [x] Re-scoped (2026-10-31): A326 is within half a unit of Excel's 15th digit (it was on main already); the other
      five cells (the two t1049-chisqinvrt cells, gen-m9 A371, A108 and A29; the A27 pair went to T-3272) are
      Excel's landing point in the flat band of its double cdf, the open rule of T-3146, which now lists them. No new
      baseline line.
- [x] No statrs search left for the gamma quantile (`rg inverse_cdf_within` finds nothing), with bounded iteration on
      untrusted input: at most 129 cdfs in one budget of `MAX_TERMS` terms (huge shape or scale, p near 0 and 1).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The five cells and 49 ulp lines of decision 0152 (now T-3146, T-3238 and T-3239, after T-3016); the CDF kernel (T-0971 follow-ups T-2367, T-2638).

## Outcome (2026-10-31)
`gamma_quantile` (`incomplete_gamma/mod.rs`) no longer starts from statrs' `Gamma::inverse_cdf` search (halve and
double, 8 bisections, 4 Newton steps, up to about 2,120 cdfs). Its refinement (`special::refine_increasing`) starts from
Wilson and Hilferty's a (1 - 1/(9a) + z/(3 sqrt a))^3 (`wilson_hilferty_start`, z by statrs' normal closed form), or
from the whole range where that is not a positive finite number. `INVERSION_CDFS`, which the lifting table charges an
inversion by, is 129 (was 2,250). Without any start (64 halvings of the whole range) the inversions near Temme's shape
would spend their term budget sooner; with this one `gamma_quantile(2E6, 1, 1E-300)` still reproduces p, and
`GAMMA.INV(0.5,1E7,1)` is NaN (`#NUM!`) as on main (DEVIATION in statistics.md).

Eval (`make eval` on main 1248554ae and on this branch, excel-graph): four cells moved, none regressed.
`make eval-baseline` without `--allow-regressions`: ironcalc-calc 394 -> 393 (GAMMA.xlsx F7 now Excel's
1.9999999999999996). gen-m9 ecc87f80077959e0 A231 `CHISQ.INV.RT(NUMBERVALUE(...))` went from visibly off to within 15
digits (Excel 0.3718455498968906, ours ...9045, was ...9034). gen-m9 35e8168aae2b00f8 A196 and 4bad303d32cc3f46 A289
`GAMMAINV(0.5,...)` went from 1 ulp under Excel's 3.6720607488508965 to 1 ulp over. The doubles next to a crossing
are not always monotone in the accurate cdf, so the start can move the landing by an ulp.

Criterion 1: a 60-digit mpmath check puts every remaining cell in the flat band T-3146 describes. At Excel's double
the true cdf of the tail it is given is within 0.73 ulp of p, while ours is within an ulp of the true quantile:
`CHISQ.INV.RT(0.999,5)` -44 ulp (cdf +0.13), `(0.999,20)` -72 (+0.71), `CHISQ.INV.RT(0.9999,3)` +3,084 (-0.69),
`CHIINV(0.975,5)` inside A108 +10 (-0.65), and A29 at the correctly rounded `T.DIST(6,6,TRUE)` -70 (-0.47).
T-3146's two attempts found no search that lands there, so these five go to T-3146, listed in its "Also owned: T-3131's
cells". A326 `CHISQ.INV(0.975,3)` was already within 15 digits on main (Excel +3.5 ulp, cdf +0.64; ours -0.5).

