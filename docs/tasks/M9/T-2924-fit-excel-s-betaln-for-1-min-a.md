---
id: T-2924
title: "Fit Excel's betaln for 1 < min(a, b) < 8 < max(a, b): a stored double -3 to +2 ulps off the accurate ln B (probes/t2829-ftest-ln-beta-grid)"
milestone: M9
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-2829]
campaign: visible
---

## Goal
A form for Excel's ln B(a, b) with both shapes above 1 and one above 8 (F.TEST's and BETA.DIST's bpser) that
reproduces the integer-ulp map of probes/t2829-ftest-ln-beta-grid on at least 90 of its 99 cells, or the forms tried
recorded in docs/excel-semantics/statistics.md.

## Context
T-2829 read Excel's L = ln B off 99 F.TEST cells deep in bpser (x near 1E-12, z unstored), a from 2.5 to 8 by b from
8.5 to 19.5: `python docs/research/t2829/ln_beta_map.py` (output in `ln_beta_map.out`). Every cell's L is a stored double
an integer k ulps from fl(accurate ln B), k from -3 (a = 6, b = 11.5, gen-m9/b6593d7dd13f2dac A306) to +2; fractions of
L + ln a, L + ln b or L + lnGamma(a + b) are random, so the stored quantity is L itself. Scores of the forms tried (cells
reproduced of 99): accurate 45 (what `f_test_tail::neg_ln_beta` ships above 8), TOMS betaln with Excel's LN 49, the TOMS
reduction with w = prod a/(a + b) 55, gamln(a) + algdiv(a, b) unreduced 40, Excel GAMMALN sums in each order 15-29.
|k| <= 1 at a = 2.5, 4, 4.5 and 5, up to 2 at a = 3, 3.5, 6, 7 and 8, and 3 only at (6, 11.5); for integer a (common
a0 = 1 after the reduction) it is not constant in b, so the error is not algdiv(a0, b) alone. The same L is open for BETA.DIST (statistics.md, "algdiv
OPEN": Excel's algdiv about 1 unit from TOMS's), so a fit should be checked on BETA.DIST's cells too.
Ideas: the reduction product in x87 or in another order; lnw from a product of quotients h/(1 + h) with LN of each
factor; Stirling's series for lnGamma(a + b) and lnGamma(b) with Excel's own constants; a probe at a = 1.5 .. 2 to pin the
unreduced pieces (k at a = 2.5 is -1 to +1).

## Acceptance criteria
- [ ] A candidate scored on all 99 grid cells and on BETA.DIST's bpser cells with a shape above 8 (`uv run rustsheet-fitlab`
      or the script above); the best form and its score in statistics.md.
- [ ] If a form reproduces at least 90 grid cells without losing a bit-exact F.TEST or BETA.DIST cell, a follow-up task to
      port it to `f_test_tail.rs` / `bratio`.
- [ ] `make check` green; reviewers approve.

## Notes from the first run (2026-10-27, time-boxed, in progress)
Scored on the 99 grid cells, with each cell's k computed as in `docs/research/t2829/ln_beta_map.py` (scripts kept in
scratch, not committed):
- LN of a GAMMA product, using Excel's own GAMMA cell values (probes `=GAMMA(v)`, all 242 points on file, within 0.33 ulp
  of exact at 6, 11.5, 17.5, 19, 27.5): LN((Ga·Gb)/Gab) 47, LN(Ga·(Gb/Gab)) 47, LN(Gb/Gab) + LN(Ga) 56, and
  LN(Ga) + LN(Gb) - LN(Gab) 27-34. Correctly rounded GAMMA gives 45-57. These are at noise level, so this is not the form.
- Size argument: Excel's L misses by up to 2.5 ulp before its last rounding (5E-15 absolute at a = 6..8). TOMS's
  reduction pieces (w, lnw, gamln on (1, 2], algdiv's u and v) are all below 3 in magnitude, so in doubles they cannot
  make that error. The error needs a term of magnitude about 20-40, such as lnGamma(a + b) or (b - 0.5)·ln b. However,
  Excel's GAMMALN cell values used in a sum score 15-29. Next: Stirling-type pieces such as (a + b - 0.5)·LN(a + b), with
  Excel's LN and with the cancellation left to the sum. Also algdiv's d·alnrel(a/b) and a·(LN b - 1) taken from
  x87-stored partial sums, and an inline lnGamma inside betaln that differs from the GAMMALN worksheet function.
- Stirling forms with a left unreduced score worse: lnGamma(a) + (b - 0.5)LN b - (a + b - 0.5)LN(a + b) + a + (D(b) -
  D(a + b)), with D the exact Stirling remainder, scores 12-22 in four orders. lnGamma(a) + TOMS algdiv(a, b) pieces
  (w - u - v, using Excel's LN) scores 34-35. Excel's misses are modest: |k| = 0 at 45 cells, 1 at about 45, 2 at 8 and 3 at 1,
  so it looks like a pre-rounding error of about 1 ulp. A term of magnitude 20-40 rounded once fits that, and a form
  with cancellation across several such terms does not.

## Out of scope
- Porting the form (its own task), the shapes of 1/2 and 1 (the remainder model), min(a, b) < 1.
