---
id: T-2586
title: "Crack the densities left open by T-2513: Loader's constant and bd0 series, the T constant, BINOM's interior pmf, NEGBINOM's final step, the F density"
milestone: M9
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Bit-level models, scored on held-out probes, for the density pieces T-2513's round 3 left open. The engine tasks T-2507,
T-2585 and T-2146 then have targets for the cells they cannot reach.

## Context
- docs/excel-semantics/statistics.md entries "Beta density for min(a,b) > 1 is Loader's saddle point", "Student t
  density is Loader's dt", "BINOM.DIST pmf: closed forms at k = 0 and k = n", "NEGBINOM.DIST pmf sits on the shifted
  binomial kernel" and "F density is not the beta density" (all T-2513). Work in docs/research/t2464/ (new `r4-*`
  directories), importing r3's libraries (`r3-loader-bd0/r3lib.py`, `r3-loader-const/r3lib.py`, `r3-tdensity/tdlib.py`,
  `r3-binom-pmf/blib.py`, `r3-fdensity/flib.py`); verifier reports in `verify-r3-*` override worker claims.
- Open items, largest first:
  1. **Loader's constant residual** (BETA.DIST, a, b > 1): 2-4 units sd after accurate stirlerr, Excel's LN pieces and
     stored k, n, nk; an n trend on the half-integer grid (+1.7 at n < 8 to -0.4 at n 22-31); -10 to -14 units at n
     70-105; noise at min(k, nk) < 3. Next steps: r3-loader-const section 8 (ln 2π as a long-double constant, LN(n+1)
     against log1p(n), single-ulp sweeps with every piece in (35, 120), stirlerr of small non-half-integer v).
  2. **bd0's series branch**: the +δ(fl(2kv)) term has the opposite sign of every natural formula, and 3-41 units of
     structured noise remain with n·x exact; the series and log-form branches differ by a shape-dependent offset.
     r3-loader-bd0 section 5 (double-double 2kv, R >= 4.1's ebd0, log1pmx). The T density is a one-parameter probe of
     it (k = n/2, M = (n + t²)/2, d = -t²/2 exact).
  3. **The T density's constant**: Excel is ±1-2 units off the exact Loader constant per df, and it is not
     r3-loader-const's stirlerr table (verifier: exact constant 573 against 501 of 960 on hold2, 505 against 501 of
     1,000 on its hold-out). Also open: the denominator order near the mode (sw3 against sw2) and very large df
     (`T.DIST(±10,1E6,FALSE)` -46 ulp).
  4. **BINOM.DIST's interior pmf**: exponent extended, EXP once, but lf/2 and the stored chain are not identified
     ((3,8) fits only with h' = h_exact - 1.85 units), plus unexplained noise of 11-115 units at (13,30), (30,100),
     (150,1000) and poor large-n cells. Best hold-out 2,006/8,234/8,917 of 10,368 against CR 2,021/6,951/8,031.
  5. **NEGBINOM.DIST's final step** on dbinom_raw(r-1, f+r-1, p, q): fl(p·B) is refuted at the bit (verifier vneg1.out:
     168 of 384 at r = 1 on the hold-out).
  6. **The F density**: the form of the x-power term ((a-1)·ln x is no stored Excel LN product), the large-shape noise
     (sd 20-200 units on a [32, 64) lattice at (100, 100)), the +3 to +5 unit offset at d1 = 1, d <= 2 cells, and
     non-integer df (truncation). r3-fdensity section 5.
- Not here: algdiv's lnB for min < 1 < 8 < max (T-2512 owns it, with r3-m1-residual's pins), GAMMALN below 1 (T-2122).
- Rules: positive controls before trusting a method; a real hold-out (a probe not used for any choice); record
  negatives (decision 0056); exact-piece literals (Excel truncates formula literals to 15 digits).

## Acceptance criteria
- [ ] Once the engine tasks that use the T-2464/T-2513/T-2586 data have landed, prune research outputs that only
      intermediate scripts read (`*.out`, `*.meta.json`, derived `.tsv`) per decision 0101, keeping the eval cases,
      RESULTS.md files and the scripts that reproduce the statistics.md counts.
- [ ] Each item above has a RESULTS.md with a model scored on a held-out probe, or the negatives tried with their
      counts.
- [ ] New probes are committed under eval/suites/probes-numeric/cases/ with their sidecars.
- [ ] statistics.md's T-2513 entries are updated with what is found.
- [ ] Engine follow-ups are filed for any solved piece, or the owning engine task's criteria are amended (T-2507,
      T-2585, T-2146, T-2219).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
- The cdf pieces (T-2512).

## Progress
- 2026-10-24, round 4 (F density only; docs/research/t2464/r4-f-brcomp, r4-f-power, r4-f-small-df, verify-r4-*):
  TOMS brcomp rejected; the stored-product structure ln F = T + P - ln x and the side-dependent Stirling-form
  constant found; best model M-sd 314/919/1,029 of 1,344 on an untouched hold-out (CR 104/688/882), not bit-exact and
  not at visible parity. statistics.md "F density is not the beta density, R's df or a GAMMALN formula" has the
  details. Next: combine M-sd's constant with r4-f-power's stored sum S2, pin the -h·L(r) term for r <= 0.375, and
  the large-shape constant (probe x = 1 ± k ulp on a dense min(d) >= 16 grid).
- 2026-10-24, round 5 (r5-f-combine, r5-f-smallr, r5-f-bigconst, r5-f-edges, r5-f-final, verify-r5-*): the argument
  and range rules are solved (handed to T-2095); M-final is the best model (126/522/722 of 1,584 on its untouched
  hold-out, CR 63/269/444) but not at visible parity. The blocker is the power term P = c·ln x (10% within 15 digits
  at |P| >= 256) and a ±1.5-unit scatter at exact-product shapes. Next: Excel's exponential step for this density
  (exp2 split, x87 F2XM1/FSCALE, value-space x^(a-1)·(1+r)^-h with per-factor pow roundings), probed at (2,2) side 1
  where E = LN(x) - 2·LN(fl(1+x)) has no other rounding.

## Decision 0188 note (2026-10-31)
Owner's 2026-10-31 rulings, R5: the densities this task is cracking (BETA.DIST, F.DIST, BINOM, NEGBINOM last digits)
are Phase 1b work. Its status is unchanged and its open PR may still merge; the remaining cells, all dense probes-numeric
sweep cells within Gate A's envelope, count as Phase 1b evidence under R1(c), not toward the Phase 1 exit. Follow-ups it
files for those cells carry `defer: phase-1b`.
