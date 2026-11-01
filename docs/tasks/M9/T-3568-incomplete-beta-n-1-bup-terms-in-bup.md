---
id: T-3568
title: "Incomplete beta: n > 1 bup terms in bup+bgrat, bup's arithmetic and the a0 < 1e-15 band"
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
bup's terms and arithmetic inside L140's bup+bgrat are modelled, for n > 1 and for the F, BINOM and NEGBINOM callers.

## Context
- Split from T-3468 (size; one task per open item), itself split from T-3383 and T-2512. T-3383 pinned the density's
  lnB at min < 1 < 8 < max on 156 of 160 shapes (docs/research/t2464/r4-algdiv/RESULTS.md). Work in
  docs/research/t2464/ in a new `r5-*` directory, importing r2-bratio's `bratio_excel.py`, r2-tdist's `xlk.py`,
  r3-bgrat's `data.py`/`bscore.py` and r4-algdiv's `scan.py`. Method: docs/research/excel-kernel-playbook.md.
- Rules: positive controls before trusting a method; a real hold-out; record negatives (decision 0056); truncate
  formula literals to 15 digits as Excel does.
- **n > 1 bup terms** (2 <= b < 40, a > 15, x > 0.7) and the F, BINOM and NEGBINOM callers of L140 bup+bgrat: probe
  them and check that RN(W + D) holds, with W the n-term bup on Excel's density in bup's orientation.
- **bup's arithmetic in bup+bgrat** (T n <= 30: 356 of 1,344 bit-exact) and the a0 < 1e-15 band at
  0.04 <= y < 0.25 (Excel 1e-14 to 3.8e-13 relative off the true value, a problem in bgrat's incomplete-gamma ratio).

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
docs/research/t2464/r5-bup/RESULTS.md. Solved for n >= 2 at the cell level, with a leftover at n >= 10. L140
bup+bgrat with n bup terms is RN(fl(ret · S) + D):
- ret = x87(P·fl(x0·y0)/c), with P Excel's density at the lowest shape c in bup's orientation (r3-bgrat's n = 1 form);
- S = 1 + Σ fl(R_i · EXP(fl(i·LN(y0)))) in doubles. bup's powers of x go through Excel's EXP, and TOMS's recurrence
  in any arithmetic does not reproduce Excel.

Scores (exact/4 ulp/15 digits):
- Frozen before scoring (commit c8cee195a): hold-out 394/1,163/1,383 of 1,650 and a post-freeze sweep 186/574/696 of
  825.
- TOMS's recurrence on the same density: 139/742/1,125 and 68/382/583. The research kernel: 102/575/892 and
  56/286/450. CR: 33/195/389.
- BINOM, NEGBINOM and F take the same composition at their oriented call (at c = 1 with D = x0^a0).

What is left:
- A residual at n >= 10 grows with the binade of (n - 1)·|ln y0| (median 5-9 units there).
- The form does not carry to the min <= 1 bup+bgrat branch: T at n <= 30 gives 411 against 412 for the recurrence,
  and the exact sum stays best at 598 of 1,344.
- The a0 < 1e-15 band was not attempted.

Probes (probes-numeric): t3568-r5-bup-tune, -hold, -fresh, -terms-tune and -terms-hold (24,240 cells, excel-graph).
Engine follow-up: T-3712 (depends on T-3382). Eval delta: none in gated suites (research only). The five new
probes-numeric cases add cells the engine misses: column A is visibly off on 1,403, 1,371 and 672 cells of tune,
hold and fresh.

