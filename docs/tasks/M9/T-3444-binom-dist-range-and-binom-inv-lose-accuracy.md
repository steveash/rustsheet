---
id: T-3444
title: "BINOM.DIST.RANGE and BINOM.INV lose accuracy at a large n (statrs past the walk budget)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
BINOM.DIST.RANGE and BINOM.INV (and CRITBINOM) at n from 6E5 to 2^31 - 2 are within 15 digits of Excel, as BINOM.DIST
is since T-3352, instead of grossly wrong.

## Context
- Found by T-3352: past the double-double walk's 2^10 terms (`discrete.rs` `binomial_range`) BINOM.DIST.RANGE falls back
  to statrs' cdf difference, and BINOM.INV's bisection crosses statrs' cdf throughout. Near the mean at a large n statrs'
  cdf is grossly wrong: `BINOM.DIST.RANGE(1000000000,0.3,300000000,300000100)` is 1.47 (a probability above 1) and
  `BINOM.INV(1000000000,0.3,0.5)` 299998246 (the median is 300000000). statistics.md "BINOM.DIST and NEGBINOM.DIST at a
  large n: basym and Loader, within 1E-11 of the math (T-3352)".
- BINOM.DIST's cdf now takes the bratio port with basym (`binom.rs` `binom_dist_of`, `bratio::basym_tails`), and its pmf
  past the walk Loader's kernel (`binom.rs` `loader_pmf`). Reuse them; Excel's form for BINOM.DIST.RANGE (a cdf
  difference or a sum of pmfs) and BINOM.INV's search are not known: probe them first (excel-probe).
- `binom_inv_of`'s 31-step bisection and its lifting weight must stay bounded (directive 5).

## Acceptance criteria
- [ ] A probe of BINOM.DIST.RANGE and BINOM.INV at n 1E4 to 2E9 (oracle-filled) is within 15 digits of Excel, with the
      right type and error.
- [ ] Gate A floor on a fresh sweep (decision 0188 R2): 0 points outside both 15 digits of Excel and 1E-9 of the math.
- [ ] No gated cell regresses; the rule is in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- BINOM.DIST and NEGBINOM.DIST (T-3352).
