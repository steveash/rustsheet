---
id: T-3131
title: "Gamma quantiles: T-0972's remaining visibly-off cells and its statrs search start"
milestone: M9
lane: functions-stat
status: in-progress
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
- [ ] The eight cells above are within half a unit of Excel's 15th significant digit, with no new baseline line (or
      decision 0083's allowance).
- [ ] No statrs search left for the gamma quantile (`rg inverse_cdf_within`), with bounded iteration on untrusted
      input (huge shape or scale, p near 0 and 1).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The five cells and 49 ulp lines of decision 0152 (now T-3146, T-3238 and T-3239, after T-3016); the CDF kernel (T-0971 follow-ups T-2367, T-2638).
