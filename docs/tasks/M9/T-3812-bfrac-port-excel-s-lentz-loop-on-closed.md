---
id: T-3812
title: "bfrac: port Excel's Lentz loop on closed-form terms (T-3571 model)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: accuracy
---

## Goal
The engine's bfrac reproduces Excel's arithmetic: TOMS 708's continued fraction run through Numerical Recipes' modified
Lentz loop on closed-form terms, with Excel's λ, as docs/research/t2464/r5-bfrac/model.py freezes it.

## Context
- T-3571 (docs/research/t2464/r5-bfrac/RESULTS.md, statistics.md "Incomplete beta bfrac is TOMS's fraction in a Lentz
  loop"): on Excel's own density the model scores 1,158/1,630/1,632 of 1,632 on probes-numeric t3571-r5-bfrac-hold and
  694/1,177/1,188 of 1,200 on t3571-r5-bfrac-fresh, against 162/977/1,384 and 86/532/844 for the TOMS recurrence the
  engine runs today (`crates/rustsheet-engine/src/functions/statistical/bratio/bfrac.rs`, T-2508).
- The model, line by line: `model.py` `fraction` and `cdf`. λ = fl(a - fl((a+b)x)) in the unswapped call for a > b
  as well (TOMS takes (a+b)y - b there).
- The evidence covers only a, b in [40, 5000] on the unswapped call (x below the mean, z from -8 to -0.2 sd). The Lentz model is a structural choice plus one fit: EPS = 10·DBL_EPSILON, tagged
  `/// FITTED(T-3571, 1536 cells, a, b 40..5000, unswapped call (x below the mean), z -8..-0.2 sd)` at the constant (decision 0188 R4.3). Gate the whole Lentz path to that region; everywhere else in bfrac's
  region (a0 < 40 with b0 >= 40, a or b above 5000, the swapped call, within 0.2 sd of the mean) keep today's TOMS
  recurrence, the accurate path, and say so in a comment. TINY = 1e-30 is NR's FPMIN (not a fit).
- The engine's density (T-3244) is not Excel's at large shapes, so engine bit-exact counts stay below the model's
  (which reads Excel's density off the sheet); score both with the engine's density and record them.

## Acceptance criteria
- [ ] `bfrac.rs` evaluates r with the Lentz loop, b0 = c·(a/(a+1)), α = v/s/s·u·n·(b-n)·x·x, β = n + fl(n·x·(b-n))/s +
      ((a+n)/(a+2n+1))·(c + n·(y+1)), stop |C·D - 1| <= 10·DBL_EPSILON, cdf = fl(brc·f), on the gated region only, with the FITTED tag
      line above; the term bound and `None` path stay (directive 5); outside the region the TOMS recurrence is unchanged.
- [ ] Unit tests pin a few cells of t3571-r5-bfrac-hold where the engine's density equals Excel's.
- [ ] Eval: probes-numeric t3571-r5-bfrac-* and t2464-w2f-xruns-1's bfrac cells counted before and after in the
      Outcome; no eval regressions (or a decision record per decision 0082 for cells lost to the better model).
- [ ] Gate A envelope rechecked with the engine's density on probes-numeric t3571-r5-bfrac-fresh (T-3571's 1,200 of
      1,200 used Excel's density read off the sheet), every point outside listed; once BETA.DIST has a Gate A table
      (T-3529), an `r4-T-3812` batch in `eval/accuracy/tables/BETA.DIST/` instead (decision 0188 R4.2).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The open per-term rounding past 10 terms (T-3813); the density itself (T-3244); basym (T-3322).
