---
id: T-2633
title: "BETA.DIST/BETA.INV after T-2119: BETADIST A159 and four probes-numeric cells visibly off, nine chained BETA.INV cells 1-4 ulp"
milestone: M9
lane: functions-stat
status: in-progress
priority: P1
size: S
tier: opus
campaign: accuracy
depends_on: []
---

## Goal
The BETA.DIST cells that are still visibly off after T-2119 reach visible parity. The nine `BETA.INV(BETA.DIST(...))`
cells that T-2119 moved from bit-exact to 1-4 ulp match Excel to the bit again. BETA.DIST's cdf stays on
`incomplete_beta`.

## Context
T-2119 put BETA.DIST's cdf on `incomplete_beta::beta_reg_complement` and refined BETA.INV to the crossing of that cdf
(`special::refine_increasing`). That fixed 27 gated lines, and recorded these regressions under decision 0083's
standing allowance (all numeric, none visibly off). Excel against ours in `ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx`
`Sheet1`:
- H2: 3.0 vs 3.000000000000001 (2 ulp)
- H23: -0.23399999999999987 vs -0.23399999999999999 (4 ulp)
- H42, H44, H46, H47, H48, H49: 0.23399999999999999 vs 0.234 (1 ulp)
- H67: 3.0 vs 2.999999999999999 (2 ulp)

Visibly off after T-2119 (the visible-parity stage):
- gen-m9/35e8168aae2b00f8 Calc!A159 `BETADIST(1E-10,0.5,10000000000.5)`: Excel 0.8427007929652814, ours
  0.8427009368248534. That is closer than statrs' 0.8427190549884069, but still off in the 7th digit. A shape of
  1E10 is the `beta_reg_complement` prefactor's huge-b case.
- Ungated probes-numeric, newly visibly off with T-2119:
  - t0969-beta-prefactor Pref!D85: Excel 1.6851260494385718E-11, ours ...776E-11 (18 ulp).
  - t0969-beta-prefactor Pref!D86: Excel 8.417666830127187E-07, ours ...7222E-07 (33 ulp; bit-exact on statrs).
  - t0969-t-beta-link Link!E56: Excel 9.114921170875367E-05, ours ...372E-05 (4 ulp).
  - t0969-t-beta-link Link!F6 (`=E6/2`): Excel 0.07797913037736931, ours ...694 (6 ulp).

  See statistics.md, the t0969-beta-prefactor paragraph, and T-0969 attempt 9 for the (a near 7.5, small x) region.
- Ungated probes-numeric, newly 1-2 ulp with T-2119 (58 cells: 44 at 1 ulp, 14 at 2):
  - m9-beta-inv 12, t1745-beta-inv-roundtrip 21, t0969-t-beta-link 20;
  - m9-beta-dist-a0p5, -a2, -a5, t0969-beta-prefactor and t1543-inverse-cdf-tiny-p, 1 each.

- Already visibly off, and further off with T-2119:
  - t1543-inverse-cdf-tiny-p A73 `BETA.INV(1E-300,1,1)` (144 -> 344 ulp), A74 (152 -> 196), A75 (87 -> 111) and A82 (4 -> 6);
  - m9-beta-dist-a50 A1 (31 -> 54), m9-beta-dist-a0p0001 A3 (9 -> 11), t0969-beta-prefactor D62 (5 -> 7);
  - BETADIST_BETAINV G5 (133 -> 150).
- probes/stat-quantile-lift E9 `BETA.INV(1-2^-53,1.0024017013258346,12177.162026553196)`: `#NUM!` on main, now
  0.0030131411834558195. Excel gives 0.0691959487211844 (T-1871 owns the fit).

- Added by T-2506 (L140 bup + bpser, 2026-10-27, decision 0083 allowance), `BETADIST_BETAINV.xlsx` `Sheet1`, 1 ulp
  each: G15 0.4999999999999999 vs 0.49999999999999994, G65 0.39911862276339233 vs 0.3991186227633924, H32 (BETA.INV
  on G32) 0.555 vs 0.5550000000000002.

These cells passed before only because statrs' cdf and its inverse were wrong in the same way (T-2119 Context). The
H42 group follows G42 (BETA.DIST), which is still 4 ulp from Excel. So fitting G42's last bits probably moves H42-H49
too. Code: `functions/statistical/beta.rs`, `inverse_beta.rs`, `special::refine_increasing`. See
docs/excel-semantics/statistics.md "BETA.INV solves Excel's own CDF to the crossing (T-1745)".

## Acceptance criteria
- [ ] Visible parity: gen-m9/35e8168aae2b00f8 A159 and the four probes-numeric cells above are within half a unit of
      Excel's 15th significant digit.
- [ ] The nine lines above leave `eval/baseline/ironcalc-calc.txt`, with no new known-failure line in any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETADIST_BETAINV G5 and G26 (the T-1044 sweep).

## Stage (decision 0114)
`campaign: accuracy`: a 50-digit reference shows Excel is close to correctly rounded for this family and the engine is not. Aim for a correctly rounded kernel, scored against an mpmath reference and the oracle; do not fit Excel's last bits. Cells left within 4 ulp after that go to a `defer: phase-1b` follow-up.
