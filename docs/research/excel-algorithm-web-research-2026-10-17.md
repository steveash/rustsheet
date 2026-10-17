# Excel statistical-kernel algorithms: what is public, and what the fit lab made of it (T-1758, 2026-10-17)

Owner request: check the web for what people know about the algorithms behind the
distributions that still differ from Excel by a few ulps, then test any concrete claim in the fit
lab. Two research passes covered the normal/erf family and the beta/t/F/gamma family.

## What is public

**Nobody has published the algorithms.** Microsoft names none. No paper, forum thread or blog
post found reverse-engineers Excel 2010+ at the last digit. The accuracy studies measure error
without naming methods: Yalta 2008, McCullough & Heiser 2008, Knüsel 1998/2005/2011, Mélard
2014. The fit-lab findings in docs/excel-semantics/statistics.md already go further than
anything public.

What Microsoft does document (support.microsoft.com, "What's new: changes made to Excel functions";
the 2010 developer reference; KB 826772/828313/828888):
- **Changed in Excel 2010 for accuracy, methods unnamed:**
  - the normal family: NORM.S.INV, NORM.INV, NORM.S.DIST, ERF;
  - the beta, gamma and binomial family: BETA.DIST, BETA.INV, BINOM.DIST, CRITBINOM, CHI*, F*,
    GAMMA.*, GAMMALN, T.DIST, T.INV.
- **The inverses are iterative searches on the matching DIST function.** This covers NORM.S.INV,
  BETA.INV, T.INV, F.INV, GAMMA.INV and CHISQ.INV. The docs say "precision depends on the
  precision of the DIST function", and the search returns #N/A after 100 iterations.
- **Search tolerance:** Excel 2002 and earlier stopped at |CDF − p| < 3E-7. From Excel 2003 the
  search runs "to the limit of Excel's precision" (KB 828313).
- **`Workbook.AccuracyVersion`** keeps the Excel 2007 and 2010 algorithm generations. "Latest" (the
  default) may have changed again after 2010.

Third-party relatives (none claims bit-exact Excel parity):
- **LibreOffice** `GetBetaDist`, in sc/source/core/tool/interpr3.cxx:
  - Excel-like special cases for b = 1 and a = 1, though it uses pow and expm1;
  - reflection at the mean;
  - A&S 26.5.8's continued fraction;
  - a pdf·x·y prefactor for a, b > 1;
  - a Boost Lanczos sum (g = 6.0246800407767296).
- **LibreOffice/Gnumeric/R** use AS 241 for the inverse normal and Cody's erfc for the normal CDF.
- **jStat/formula.js** use Numerical Recipes' betacf.
- **VBA "Excel NORMSINV" ports** use Acklam or AS 111 plus a Newton step.

## Hypotheses tested

| hypothesis (source) | result |
|---|---|
| NORM.S.DIST is Cody's erfc (R `pnorm_both`) | refuted: 991 of 1,779 cells, against 1,103 for glibc's `0.5*erfc(-x*sqrt(0.5))` and 881 for the engine (fit `norm-s-dist`) |
| NORM.S.INV's centre is AS 111 (split 0.42), with or without Newton polish, on Cody or glibc CDFs | refuted. Central cells, of 408: AS 111 alone 12; AS 111 plus 1 or 3 Newton steps 70–95; the best polish overall is PPND16 plus one Newton step on Cody, 114. statrs gets 164 (fit `norm-s-inv`) |
| NORM.S.INV iterates on Excel's own NORM.S.DIST (Microsoft docs) | supported in part. Probe t1758-norm-s-inv-roundtrip: Excel's central x is a crossing of its own CDF at 234 of 340 points; the CDF is flat at 36; 70 miss by 1–32 ulps of p. This is the same shape as BETA.INV (T-1745) |
| BETA.DIST is LibreOffice's GetBetaDist | ties the best candidate: 156 of 283 with Excel's closed forms (134 without), against 158 for the Cephes combination and 87 for the engine. Neither is Excel's |
| The remaining BETA.DIST error is Excel's Gamma (lnB) | refuted: Excel's relative error depends on x, not only on (a, b), at all 38 (a, b) pairs. Plugging in Excel's own GAMMALN/GAMMA values does not close it |

## General conclusions

1. **Excel's inverses are root searches on Excel's own CDF.** BETA.INV (T-1745) and the centre of
   NORM.S.INV both land on a crossing of Excel's CDF at most points, with a minority stopping a
   few ulps of p short. Inverse parity therefore follows forward parity, so the forward kernels
   (T-0968 normal CDF, T-0970 incomplete beta) come first.
2. **BETA.DIST's gap is the small-result branch.** In the reflected branch (1 − small) both
   candidates match 90 to 94% of cells, because 1 − small hides the small term's error. In the
   direct branch no candidate gets more than 18 of 110 cells, and the correctly rounded value gets
   10. There Excel's relative error grows with a·err(ln x): at x = 0.1 it is about 1.2E-14 for
   a = 100, against 1.7E-14 predicted from ln(0.1)'s double rounding. So Excel builds the
   prefactor in log space from a double ln x. The rest is the rounding of the exponent sum, and
   none of 360 tested combinations of fraction, prefactor form and operation order reproduces it.
3. **NORM.S.DIST's best stand-in is glibc's `0.5*erfc(-x*sqrt(0.5))`** (1,103 of 1,779 against
   881 today). T-0968 can take it as an interim kernel.

## Where to look next
- The 2009 Excel-team blog post "Function Improvements in Excel 2010"
  (blogs.msdn.com/excel/archive/2009/09/10/function-improvements-in-excel-2010.aspx), and the
  full Mélard 2014 and Knüsel 2011 papers. None was reachable from the agent containers.
- A direct-branch BETA.DIST probe that separates the prefactor from the continued fraction:
  BETA.DIST at b = 2 (closed form x^a·(1 + a − a·x)) and at small x where the fraction is ≈ 1.
