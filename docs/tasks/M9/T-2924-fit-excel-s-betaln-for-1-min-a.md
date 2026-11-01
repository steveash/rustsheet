---
id: T-2924
title: "Fit Excel's betaln for 1 < min(a, b) < 8 < max(a, b): a stored double -3 to +2 ulps off the accurate ln B (probes/t2829-ftest-ln-beta-grid)"
milestone: M9
lane: research
status: blocked
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

## Notes from the first run (2026-10-27, time-boxed)
Scored on the 99 grid cells, with each cell's k computed as in `docs/research/t2829/ln_beta_map.py` (scripts kept in
scratch, not committed, so these scores are unreproduced from the repo):
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

## Notes from the second run (2026-10-27, time-boxed)
`docs/research/t2924/structure.py` (output `structure.out`) rules out whole families of forms at once, instead of
scoring one candidate at a time:
- Lattice: no L = g(a) + g(b) - g(a+b) over doubles g within 2 ulp of lnGamma reaches every cell. In seven sum orders
  (double, x87 stored, one extended sum), 16 to 29 of 99 cells cannot be reached by any table. g(b) and g(a+b) are
  doubles on a grid 4 ulp(L) wide, so L's last bits cannot come from them. Any form that sums double lnGamma values
  of magnitude 8 or more is out, Excel's GAMMALN included.
- Reduction: for integer a, TOMS betaln reduces a to a0 = 1, so L = fl(LN(w) + A(b)) with A depending on b only. For
  none of 19 w variants does one A(b) fit the six integer a at more than 1 of the 11 b values. This covers
  h/(1+h) and t/(t+b) factors, double, x87-stored and extended products, both orders, and even the exact ln of the
  product. So the whole a-reduction family is out for integer a. TOMS betaln itself passes the same separable test on
  those cells.
- Separable: no L = fl(lnB + f(a) + h(b) + q(a+b)) fits, for any f, h and q (a linear program). This form covers
  every per-argument lnGamma table, in any precision, summed once. It also fails on the integer-a cells alone.
  Controls: a separable form passes and random k fails. At most 94 of 99 cells fit such an error, against 82 for
  random k, so the error is not separable but is not noise either.
- Forms scored this run, of 99 cells or of the 66 integer-a cells:
  - TOMS algdiv on the unreduced a, with alnrel as TOMS, LN(1 + h), correctly rounded log1p or LN((b + a)/b), each
    in double, x87 or extended, plus GAMMALN, correctly rounded, TOMS or extended lnGamma(a): 44-47 of 99.
  - L = LN(w) - LN(b) and LN(w/b), with w the products above: 36 of 66 (accurate 27 of 66).
  - ln((a - 1)!) - sum of LN(b + i), in both orders, double, x87 or extended: 38 of 66.
  - Excel GAMMALN sums: 16-28 of 99. Correctly rounded lnGamma sums: 27-34 of 99.

## Blocked because: time-boxed (decision 0056)
- BETA.DIST's bpser cells with a shape above 8 were not scored in either run; there is no negative result for them.
- Two runs. The best score is still TOMS's a-reduction with w = prod a/(a + b), at 55 of 99. Accurate ln B (shipped)
  scores 45. Every candidate is listed in the two Notes sections above.
- What is left: Excel's L carries error that depends on a and b jointly, more finely than a double lnGamma grid. For
  integer a it is not ln(w) plus a function of b. The candidates are a jointly computed form, such as a Lanczos or
  Stirling ratio in (a, b) evaluated in x87, or an EXP/LN round trip of B itself.
- Reopen on a new hypothesis recorded here, such as Excel's ln B on a denser grid. One example is a fixed b over
  a = 2, 3, ..., 8 at b = 8.5 to 9.5 by steps of 1/16, to see whether k varies smoothly in b (a formula in a/b) or
  jumps (rounding of a big term). Excel's BETA.DIST bpser cells with a shape above 8 are another source.
- Gap triage 2026-10-28: that dense-grid probe is filed as T-3015 (research), through BETA.DIST's bpser since F.TEST
  cannot give fractional shapes. Its answer is the new hypothesis this section asks for.

- Gap triage 2026-10-30: `make unowned-off ALL=1` listed six visibly-off cells with no open owner. They are
  `2*F.DIST.RT(1E+12,df1,df2)` with df2/2 = 6 and df1/2 = 8.5 to 13, inside this task's 1 < min < 8 < max band. T-3007 is
  done and named them, and T-3015 refers to them only through T-3007. mpmath shows Excel itself is 9 to 119 ulp from the
  exact tail on them (ours 3 to 153), so they need this fit, not more accuracy. This task owns them:
  probes/t2829-ftest-tail-over-b S!A1, S!A3, S!A4, S!A6, S!A7, S!A9.

## Out of scope
- Porting the form (its own task), the shapes of 1/2 and 1 (the remainder model), min(a, b) < 1.
