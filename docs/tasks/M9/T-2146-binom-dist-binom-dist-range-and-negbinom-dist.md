---
id: T-2146
title: "BINOM.DIST, BINOM.DIST.RANGE and NEGBINOM.DIST last digits: fit Excel's evaluation (70 ironcalc-calc BINOM.xlsx known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2145]
---

## Goal
BINOM.DIST, BINOMDIST, BINOM.DIST.RANGE, NEGBINOM.DIST and NEGBINOMDIST give Excel's doubles. T-0927's residue in
`ironcalc-calc/STATISTICAL/BINOM.xlsx` has no open owner today: T-2055 lists the binomial family as out of scope
and points back to T-0927, which is done.

## Context
- Needs an oracle sweep (T-2145). T-0927 tried and ruled out the exact sum, Loader's saddle-point pmf (R's
  `dbinom_raw`: 49 of 122) and `exp(a*ln x)` times the exact remaining factor, all on the cached cells alone.
- Known failures (eval/baseline/ironcalc-calc.txt, gap triage 2026-10-20): 70 lines, BINOM.xlsx sheet by sheet:
  - BINOM.DIST: 18 (15 NumericUlp, for example E3: Excel 0.23925373020405763, ours ...576; 3 NumericClose:
    E24, E32, E67, for example E24 `BINOM.DIST(2,10,0.999999,TRUE)`: Excel 4.499992001038836E-47, ours ...802E-47).
  - BINOM.DIST.RANGE: 17 (14 NumericUlp, for example E2: 0.2552983299999998 against ...999; 3 NumericClose: E10,
    E21, E23).
  - NEGBINOM.DIST: 35 (28 NumericUlp, for example E4: 0.02800000000000001 against 0.028000000000000004; 7
    NumericClose: E3, E26, E30, E37, E40, E74, E75).
  - Ungated: probes-numeric/m9-legacy-binomdist (21 failing, for example A5 `BINOMDIST(3,10,0.5,TRUE)`:
    0.17187500000000006 against the exact 0.171875) and t1048-negbinomdist (12 failing, for example A7
    `NEGBINOMDIST(3,3,0.75)`: 0.06591796875000003 against the exact 0.06591796875).
- Evidence and hypotheses (statistics.md "Discrete distributions: within an ulp of the exact sum"):
  `NEGBINOM.DIST(0,100,0.01,TRUE)` is `exp(100*ln(0.01))` in doubles bit for bit, Excel's BETA.DIST prefactor
  ("Incomplete beta: Excel's closed forms and the best candidate", T-1744). So a likely shape for the cdf is the
  regularized incomplete beta, `BINOM.DIST(k,n,p,TRUE) = I_{1-p}(n-k, k+1)`, with Excel's own prefactor. The pmf may be
  `exp` of a log-gamma sum, or a ratio. Try them in the fit lab (`cd tools && uv run rustsheet-fitlab`,
  docs/architecture/fit-lab.md). If the cdf is the incomplete beta, share `statistical/incomplete_beta.rs` (T-0970;
  T-2119 moves BETA.DIST onto it), not a second kernel.
- Code: `crates/rustsheet-engine/src/functions/statistical/discrete.rs` (T-0927's exact walk, capped at 2^10 steps
  with statrs past it).

## Acceptance criteria
- [ ] statistics.md "Discrete distributions" records the fit: each candidate's count on BINOM.xlsx's 148 numeric
      cells, on T-2145's sweep and on the two probes-numeric cases, including the candidates that lost.
- [ ] The formulation that wins is implemented. The BINOM.xlsx known-failure lines it fixes disappear from
      eval/baseline/ironcalc-calc.txt, with no new ones in any gated suite. The target is all 70 lines; say which
      ones remain and why.
- [ ] No BINOM.INV/CRITBINOM cell regresses (they search on the same cdf).
- [ ] `make check` green; reviewers approve.

## Out of scope
- POISSON.DIST (T-2055), HYPGEOM.DIST (T-0977, T-1896), BINOM.INV with probability_s 0 (T-2050).
