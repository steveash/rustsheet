---
id: T-3571
title: "Incomplete beta: bfrac's fraction arithmetic"
milestone: M9
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
bfrac's continued-fraction arithmetic is modelled as Excel evaluates it.

## Context
- Split from T-3468 (size; one task per open item), itself split from T-3383 and T-2512. T-3383 pinned the density's
  lnB at min < 1 < 8 < max on 156 of 160 shapes (docs/research/t2464/r4-algdiv/RESULTS.md). Work in
  docs/research/t2464/ in a new `r5-*` directory, importing r2-bratio's `bratio_excel.py`, r2-tdist's `xlk.py`,
  r3-bgrat's `data.py`/`bscore.py` and r4-algdiv's `scan.py`. Method: docs/research/excel-kernel-playbook.md.
- Rules: positive controls before trusting a method; a real hold-out; record negatives (decision 0056); truncate
  formula literals to 15 digits as Excel does.
- **bfrac's fraction arithmetic.** The residual is white, with sd 3.8 units at (100, 120). Try the original Fortran
  BFRAC compiled for x87, and the per-iteration quantities (w4-fractions section 7).

## Acceptance criteria
- [x] A RESULTS.md records a model with exact, within-4-ulp and within-15-digit counts on a held-out probe, or the
      negatives tried with their counts.
- [x] New oracle probes are committed under eval/suites/probes-numeric/cases/ with their sidecars.
- [x] statistics.md's T-2464 entries are updated with what is found.
- [x] Engine follow-ups are filed for any solved piece, or the owning engine task's criteria are amended.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-3382 ports the L140 one-term composition).
- The other items split from T-3468: T-3567 (bgrat increment, F at df 1), T-3568 (bup), T-3569 (algdiv), T-3570
  (bpser's lnGamma and b < 1 ceiling), T-3571 (bfrac), T-3572 (GAMMA.DIST near P = 1); one each.

## Outcome
- docs/research/t2464/r5-bfrac/RESULTS.md. Excel's bfrac is TOMS 708's continued fraction evaluated with Numerical
  Recipes' modified Lentz loop (start from 1e-30, partial (1, b0)), on closed-form terms: b0 = c·(a/(a+1)),
  α = v/s/s·u·n·(b-n)·x·x, β = n + fl(n·x·(b-n))/s + ((a+n)/(a+2n+1))·(c + n·(y+1)), stop |C·D - 1| <= 10·DBL_EPSILON,
  cdf = fl(fl(P·fl(x·y))·f) on Excel's density P; λ = fl(a - fl((a+b)x)) also for a > b. w4's sd-3.8 residual was the
  loop: it grows with the term count and is uncorrelated with the density's error.
- Model (`model.py`, frozen at 0e334d864 before the hold-out): tune 1,236/1,533/1,536 of 1,536; **hold-out
  1,158/1,630/1,632 of 1,632** (71% against tune 80.5%, inside the 10-point margin fixed in 65189a46d; CR 45/319/781,
  TOMS doubles as ported 162/977/1,384); **fresh sweep** (built and filled after the freeze, log-uniform (a, b) in
  [40, 5000]) 694/1,177/1,188 of 1,200 (CR 15/117/268, TOMS 86/532/844); Gate A envelope 1,200 of 1,200 inside;
  w2's (100, 120) run 401/998/1,024 (w4's best 185).
- Region: the evidence covers a, b in [40, 5000] on the unswapped call (x below the mean, z from -8 to -0.2 sd); the semantics entry and T-3812 are limited to it.
- Fits (decision 0188 R4): EPS = 10·DBL_EPSILON is `/// FITTED(T-3571, 1536 cells, a, b 40..5000, unswapped call (x below the mean), z -8..-0.2 sd)`, bracketed in
  (2.1e-15, 2.5e-15) by an 8-value scan; the loop family, start and term forms are structural choices ranked on tune
  (no tag). Reproducibility floor: every choice moved whole cells; nothing was fitted below 1 ulp.
- Probes: probes-numeric t3571-r5-bfrac-tune, -hold, -fresh (excel-graph, 2026-11-02).
- Negatives with counts: RESULTS.md section 3 (TOMS recurrence variants best 193/1,024; FMA masks 156; Steed 198;
  backward 160; x87 or FMA terms, other assemblies, other Lentz starts).
- Open: one rounding per term past 10 terms (bit-exact 99% under 10 terms, 20% beyond 30); the swapped call's λ.
- Follow-ups: T-3812 (engine port, functions-stat, sonnet), T-3813 (the per-term rounding, research, opus).
- Eval delta: none (research and probes only; the new probe cases have no engine-side expectations changed).
