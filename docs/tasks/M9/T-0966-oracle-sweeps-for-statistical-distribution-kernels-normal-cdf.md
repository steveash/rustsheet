---
id: T-0966
title: "Oracle sweeps for statistical-distribution kernels (normal CDF/PDF, gamma, incomplete beta/gamma, t, Weibull) and compat-alias evidence"
milestone: M9
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Oracle-filled probe suites give every statistical-distribution fitting task from the T-0903 triage
enough Excel points to fit Excel's formulation (instead of fitting blind to the few cached cells in
the upstream workbooks), and the distribution functions that have little or no Excel evidence get
some.

## Context
T-0903 grouped the 680 failing distribution cells in `ironcalc-calc` by numerical kernel. Most of
what remains after an accuracy upgrade is last-digit (NumericUlp, <= 4 ulp) disagreement, and
Excel's values are demonstrably not the correctly rounded ones (see
`docs/excel-semantics/statistics.md`: normal CDF 1 ulp high at z = 1, GAMMA(-0.5) 1 ulp off,
T.DIST.RT not 1 - T.DIST, inverses that do not round-trip). Earlier blind attempts at this kind of
fit (T-0925, draft PR #38) fixed some cells and regressed others. Decision 0026 / T-0018: such
groups wait for an oracle sweep. T-1006 (split from T-0942) covers the inverse normal (T-0925); this task covers the rest.

One spec suite per family under `eval/suites/probes-numeric` (or `probes`), built with
`rustsheet-gen --policy numeric-rules` or hand-written specs, filled with `rustsheet-oracle fill`
(excel-probe skill). Suggested grids (adjust at pickup; keep each family a few hundred cells):
1. Normal CDF / erfc (for T-0968): `NORM.S.DIST(z,TRUE)` for z in -38..8 step 0.125, `ERFC(x)` and
   `ERFC.PRECISE(x)` for x in -6..27 step 0.125, `GAUSS(z)`, a few `NORM.DIST(x,m,s,TRUE)` and
   `LOGNORM.DIST(x,m,s,TRUE)` with s != 1.
2. Normal PDF (for T-0975): `NORM.S.DIST(z,FALSE)`, `NORM.DIST(x,m,s,FALSE)` with s in {0.5, 1.2, 3, 10},
   `LOGNORM.DIST(x,m,s,FALSE)`, `PHI(z)` on the same z grid.
3. Gamma function (for T-0973): `GAMMA` and `GAMMALN` at integers 1..171, half-integers -10.5..50.5,
   x in (0,1) and near 0, negatives between poles.
4. Student t (for T-0969, T-0967): `T.DIST(x,df,TRUE/FALSE)`, `T.DIST.RT`, `T.DIST.2T` for df in
   {1,2,3,4,5,10,30,100,1000,1e6} and x grid incl. tails; `T.INV`/`T.INV.2T` on p grid incl. near
   0, 0.5, 1; a few `CONFIDENCE.T`.
5. Incomplete beta (for T-0970, T-0974): `BETA.DIST(x,a,b,TRUE/FALSE)` for a, b in
   {0.0001, 0.5, 1, 2, 2.5, 5, 10, 50, 100} and x grid; `F.DIST`, `F.DIST.RT`; `BETA.INV`, `F.INV`, `F.INV.RT`.
6. Incomplete gamma (for T-0971, T-0972): `GAMMA.DIST(x,a,b,TRUE/FALSE)` for a in
   {0.5, 1, 2, 9, 50, 500}, b in {1, 2, 1e6}; `CHISQ.DIST`, `CHISQ.DIST.RT`; `GAMMA.INV`, `CHISQ.INV`, `CHISQ.INV.RT`.
7. Weibull (for T-0978): `WEIBULL.DIST(x,k,l,TRUE/FALSE)` for k in {0.5, 1, 2, 3.7}, l in {1, 9}.
8. Evidence gaps (decision 0031, report.md "Excel evidence per function"): the legacy compatibility
   names implemented with no Excel evidence (BETADIST, BETAINV, BINOMDIST, CHIDIST, CHIINV, CHITEST,
   CONFIDENCE, CRITBINOM, EXPONDIST, FDIST, FINV, FTEST, GAMMADIST, GAMMAINV, HYPGEOMDIST, LOGINV,
   LOGNORMDIST, NEGBINOMDIST, NORMDIST, NORMINV, NORMSDIST, NORMSINV, POISSON, TDIST, TINV, TTEST,
   WEIBULL, ZTEST) and the modern ones below 25 Excel cells (F.DIST, F.DIST.RT, F.INV, F.INV.RT,
   F.TEST, CHISQ.*, HYPGEOM.DIST, WEIBULL.DIST, Z.TEST, PHI, T.TEST): an `excel-probe` case each,
   >= 25 cells incl. argument coercion and error edges.

## Acceptance criteria
- [ ] Each family above has an oracle-filled probe suite committed (no pending cells), and the
      dependent task (T-0967, T-0968, T-0969, T-0970, T-0971, T-0972, T-0973, T-0974, T-0975,
      T-0978) links it in its Context.
- [ ] Size: eight families do not fit one M PR. Split per family at pickup with `make new-task`
      (research lane), repoint each dependent task's `depends_on` to its own sweep, and close this
      task when the splits exist, so each fitting task becomes ready as soon as its sweep lands.
- [ ] New quirks visible in the sweeps are recorded in `docs/excel-semantics/statistics.md`.
- [ ] Fixtures hold only formulas and Excel values: no tenant ids, tokens or personal data, and no
      RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, set `status: blocked` with the reason (the dependents then
      wait visibly instead of being fitted blind).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing any formulation (the dependent tasks do that).
- Inverse normal (T-1006 / T-0925) and the discrete distributions (T-0927).
