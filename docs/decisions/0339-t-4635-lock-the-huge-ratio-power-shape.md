# 0339 — T-4635: lock the huge-ratio power-shape cut in the FITTED ratchet

- **Status:** accepted (2026-11-07, T-4635)
- **Context:**
  - T-4635 found Excel's rule R for GAMMA.DIST and the chi-squared functions at a huge finite y = x / beta
    (docs/excel-semantics/statistics.md "GAMMA.DIST at a huge finite x / beta (T-4635)"; docs/research/t4635). Above
    shape 1 the result is `#NUM!` where y / a * 4 overflows. Up to some shape between 1.1 and 1.11 it is also `#NUM!`
    where y^a overflows.
  - The engine needs that shape cut as a constant, `HUGE_RATIO_POWER_SHAPE_MAX` = 1.1 in
    functions/statistical/gamma.rs. Every value in [1.1, 1.11) scores every probed cell. 1.1 was taken because it is
    the largest evidenced `#NUM!` shape (probes/t4635-gamma-dist-huge-ratio-cut Lo, shape 1.1 from y = 1E282). Shape
    1.11 is a number at y = 1E306 (probes/t4635-gamma-dist-huge-ratio-fresh Cut D1). So the constant is a fit
    (decision 0188 R4, reviewer-excel-fidelity check 7 (a)).
  - R4 guards. The outcome is categorical (`#NUM!` or 1 / 0), so the bit-exact, 4-ulp and 15-digit counts equal the
    cell counts. The y^a branch was proposed from the first fill and placed on the second fill's Lo sheet (shapes
    1.01 to 1.3). R4.1: the fit lab's hash hold-out split (fitlab/kernels/gamma_huge_ratio.py, 5-point margin) scores
    rule R with the cut at 1.1 at 845/845 train and 353/353 test GAMMA.DIST cells, against 811/845 and 335/353 with no
    power branch and 555/845 and 244/353 for the correctly rounded candidate (no `#NUM!` at a finite y). R4.2: the
    full-domain log-uniform Gate A batch GAMMA.DIST/r4-T-4635, drawn after the freeze, is 200 of 200 in the envelope;
    the aimed batch GAMMA.DIST/e-T-4635 around the cut is 120 of 120 as predicted; and the Cut sheet of probes/t4635-gamma-dist-huge-ratio-fresh (shapes 1.11 to 1.14, y 1E285 to 1E306) has 16 numbers as
    the cut predicts. The structural 4 / a quotient is a model (check 7 (c)), not a tag (T-4635 Outcome has the table).
  - The tag is `FITTED(T-4635, 2 cells, shape 1.1..1.11)`.
- **Decision:** add the new tag to `quality/fitted.json` (`scripts/check_fitted.py --update`), so
  functions/statistical/gamma.rs goes from 6 to 7 tags.
- **Consequences:**
  - The ratchet now counts this fit. Removing it, or moving the cut outside [1.1, 1.11), needs new evidence.
  - Gate A gives no value to shapes strictly between 1.1 and 1.11 where y^a overflows (gamma_dist.py
    `_huge_ratio_domain`). T-4877 owns the remaining unprobed edge (below shape 1.01); gamma.rs's `huge_ratio_cuts` pins the cuts.
