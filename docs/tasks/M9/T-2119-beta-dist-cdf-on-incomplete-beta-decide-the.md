---
id: T-2119
title: "BETA.DIST cdf on incomplete_beta::beta_reg; decide the nine chained BETA.INV cells it moves"
milestone: M9
lane: functions-stat
status: blocked
priority: P1
size: S
tier: opus
campaign: visible
depends_on: [T-0970]
---

## Goal
BETA.DIST and BETADIST's cdf use the engine's incomplete beta (`statistical/incomplete_beta.rs`,
T-0970) instead of statrs' `Beta::cdf`, and the BETA.INV cells that move with it are settled.

## Context
T-0970 ported the T-1744 best combination (plus a TOMS 708 prefactor past MAXGAM) and moved
F.DIST, F.DIST.RT, F.TEST and the BETA.DIST density to it. It left BETA.DIST's cdf on statrs
because the switch, measured on that branch, fixes 16 more ironcalc-calc cells but moves nine
BETA.INV(BETA.DIST(...)) cells in `ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx` (Sheet1!H2,
H14, H42, H44, H46, H47, H48, H49, H67) from bit-exact to 1-2 ulp. That is so whether BETA.INV
inverts statrs' cdf or the new one, and also with a bisection or a 64-ulp polish on the new cdf.
These cells passed only because statrs' cdf and inverse are wrong in the same way.
The H42 group follows G42, which stays 4 ulp from Excel. See docs/excel-semantics/statistics.md
"Incomplete beta in the ironcalc-calc workbooks" and "BETA.INV solves Excel's own CDF to the
crossing (T-1745)".
Recording the regressions needs `update-baseline --allow-regressions` under a decision record
(directive 2; precedent decision 0064). An autonomous run was not permitted to run that command,
so a person, or a reviewer-approved decision, has to settle it first.
Code: `functions/statistical/beta.rs` `beta_dist_of` (TODO(T-2119)), `inverse_beta.rs`
(`beta_inverse_cdf` should invert the same cdf), the lift weights' comments.
F.INV and F.INV.RT have the same mismatch: they still invert statrs' F cdf while F.DIST uses
`incomplete_beta` (`F_DIST.xlsx` J2, J3, L2, L3 stay NumericClose). Move them in the same
change, or measure them and file a separate task.

gen-m9 sweep (gap triage 2026-10-23, T-2255). `BETADIST(1E-10,0.5,10000000000.5)` Excel 0.8427007929652814, ours 0.8427190549884069
(1 cell).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas.
Examples:
- gen-m9/35e8168aae2b00f8 Calc!A159 `=BETADIST(1E-10,0.5,10000000000.5)`: Excel 0.8427007929652814, ours 0.8427190549884069
Cells:
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A159

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: BETADIST last digits. Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas.
Examples:
- gen-m9/899f9555e67da2d6 Calc!A101 `=BETADIST(Data!A9,0.5,1.5,Data!$A8,(123456789.123^(Data!B11-Data!D4)))`: Excel 0.939680733981541, ours 0.9396807339815411
Cells:
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A101
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A150

T-0974 (2026-10-23) measured the switch together with BETA.INV refined to the crossing of `beta_reg`
(`special::refine_increasing`, the upper tail I_y(b,a) = 1 - p in y = 1 - t above p = 0.5): 45 gated lines fixed, in-scope
cells visibly off down by about 1,300, and nine BETADIST_BETAINV.xlsx regressions, all within 4 ulp (H2 2, H23 4, H42, H44,
H46-H49 1, H67 2: decision 0083's allowance). Four ungated probes-numeric BETA.DIST cells go visibly off with the switch
(t0969-beta-prefactor D85, D86, t0969-t-beta-link E56, F6; up to 3.5 units of the 15th digit, D86). The run could not
record the regressions (`--allow-regressions` was refused to it), so it shipped only the F half. T-0974's BETAINV cells
(gen-m9/589f42e67f4b4b8d A221, c18dbf21f11093f3 A400, eced4bb6a0234c9e A291) belong here.

## Blocked because
Blocked because: the `--allow-regressions` baseline run was refused by the session's permission policy.

The code is done on branch `agent/functions-stat/T-2119-beta-dist-incomplete-beta` (2026-10-23, opus worker):
BETA.DIST's cdf is `incomplete_beta::beta_reg_complement`, and BETA.INV refines statrs' estimate to the crossing of that
cdf (`special::refine_increasing`, upper tail on 1 - p above p = 0.5, as `fisher.rs` `f_quantile`); the unused
`special::bisect_unit_interval` is gone. `make eval-check` on that code:
- fixed 26 gated lines: ironcalc-calc 18, probes 5, gen-m9 3 (including gen-m9/35e8168aae2b00f8 A159);
- visibly off in gated suites: 7,326 (lock 7,703);
- 9 regressions, all in `ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx`, all numeric and still equal to Excel at 15
  significant digits: H2 2 ulp, H23 4, H42 1, H44 1, H46 1, H47 1, H48 1, H49 1, H67 2. Decision 0083's standing
  allowance covers them (conditions 1, 3 and 4; condition 2 for gated cells).
- Ungated probes-numeric: D85 and D86 of t0969-beta-prefactor and F6 of t0969-t-beta-link now differ from Excel in the
  15th digit (against a 50-digit reference ours is 1.9E-15, 8.6E-16 and 8.8E-16 relative off, Excel 1.5E-15, 3.3E-15 and
  1.9E-16, so on D85 and D86 ours is no worse). E56 stays within the 15th digit.
- `BETA.INV(1-2^-53,1.0024017013258346,12177.162026553196)` is now 0.003013141..., the true quantile, instead of `#NUM!`
  (Excel: 0.0691959487211844, probes/stat-quantile-lift E9, still a known failure). The unit test and statistics.md say so.
`make clippy` is clean and `make test` passes on the branch.
To finish: someone allowed to run it runs `scripts/eval_guard.sh target/release/rustsheet-eval update-baseline
--allow-regressions` then `make eval-baseline`, checks that the baseline diff adds exactly the 9 lines above, lists them in
the Outcome with a follow-up ulp task, and continues at the review step.

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A decision record allows exactly the chained BETA.INV lines (or they are avoided), and the
      baseline diff adds no other line.
- [ ] BETA.DIST's cdf is `incomplete_beta`'s; BETA.INV (and F.INV, or a filed task) inverts the same cdf.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETADIST_BETAINV G5 and G26 (Cephes' fraction 160 ulp off where Excel is correctly rounded;
  Excel 34 ulp from correct): the T-1044 sweep work.

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 510 in total.
- BETA.DIST: 505 cells (13-14 digits right 384, 10-12 120, 6-9 1, fewer or 0 vs nonzero 0); most in `probes-numeric/t1745-beta-inv-roundtrip.case.json` 276; `probes-numeric/m9-beta-dist-a100.case.json` 43. e.g. `BETA.DIST(0.1,1,2,TRUE)` Excel 0.18999999999999995, ours 0.1900000000000007
- BETADIST: 5 cells (13-14 digits right 5, 10-12 0, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/m9-legacy-betadist.case.json` 5. e.g. `BETADIST(0.01,2,3)` Excel 0.0005920300000000006, ours 0.0005920300000000013

Against a 50-digit reference, Excel is close to correctly rounded for this family and the engine is the inaccurate side, so an accurate kernel (not a fit) should reach this stage.

The nine chained BETA.INV cells that move from bit-exact to 1-2 ulp are exactly the trade decision 0083 allows: record them under its allowance instead of waiting for a separate decision.
