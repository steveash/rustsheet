---
id: T-3567
title: "Incomplete beta: Excel's direct bgrat increment at b != 1/2 to the bit, and F at a df of 1 through bgrat_half"
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
Excel's direct bgrat increment D at b != 1/2 is modelled to the bit, and F at a df of 1 matches Excel through bgrat at b = 1/2.

## Context
- Split from T-3468 (size; one task per open item), itself split from T-3383 and T-2512. T-3383 pinned the density's
  lnB at min < 1 < 8 < max on 156 of 160 shapes (docs/research/t2464/r4-algdiv/RESULTS.md). Work in
  docs/research/t2464/ in a new `r5-*` directory, importing r2-bratio's `bratio_excel.py`, r2-tdist's `xlk.py`,
  r3-bgrat's `data.py`/`bscore.py` and r4-algdiv's `scan.py`. Method: docs/research/excel-kernel-playbook.md.
- Rules: positive controls before trusting a method; a real hold-out; record negatives (decision 0056); truncate
  formula literals to 15 digits as Excel does.
- **Excel's direct bgrat increment D at b != 1/2, to the bit.** Best form so far is RN(F·Q(b, z_s)/Q(b, z_e)),
  246/1,881/2,357 of 2,908 (probe t2512-r3-bgrat-split). Excel sits a median 4 units of 2^-53 below the exact value.
  Next: x runs of consecutive doubles at a fixed (a, b0) to find the stored intermediate; grat1's continued fraction
  in x87 with Excel's EXP for r.
- **F at a df of 1** reaches bgrat at b = 1/2. T-2509's `bgrat_half` on F's (x, y) misses Excel
  (`FDIST(x,1,9)`: Excel 0.190947117169724, bgrat 0.19094711716972382, 7 ulp), so find what F does differently
  (statistics.md "Student t tails are one kernel call").

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
- **F at a df of 1: solved.** The T-2510 miss was F's argument form, not the kernel. Excel's F forms
  r = n/(mx) or mx/n, whichever is >= 1, passes the smaller argument as fl(1/fl(1+r)) and the larger as
  fl(r/fl(1+r)), and at b = 1/2 takes the same bgrat_half as T. Hold-out probe t3567-r5-fdf1-hold (3,578 cells,
  model frozen first): 2,787/3,419/3,463 (bit-exact / within 4 ulp / within 15 digits) against the engine's
  1,857/2,843/3,025; d1 = 1 bgrat region 991/1,000/1,000 against 298/572/659. The r >= 1 form also wins for every
  df on cells.tsv's 3,435 F cells (2,300/3,295 against 2,178/3,237 for n + mx). Engine port: T-3580.
- **D at b != 1/2: still open, negatives recorded.** Excel's D is white noise of sd 5.7-8.5 units around the exact
  value along runs of consecutive doubles. Only the stored z shows. Prefactor variants (206-210 bit-exact of
  2,908), ln x by log1p (158, 189) and grat1's CF in doubles (200) do not beat r3-bgrat's stored-z ratio (246).
  Follow-up research task: T-3582 (Excel's own Q(b, z) inside bgrat).
- statistics.md: new entry "F's incomplete-beta arguments keep r >= 1"; "Incomplete beta is TOMS 708 bratio"
  (caller arguments), "Student t tails are one kernel call" (the df-of-1 paragraph) and "the integrated model"
  (D negatives; F's df-1 item removed from Open) updated.
- Eval delta: none in the gated suites (research only; probes-numeric is ungated). New probe:
  t3567-r5-fdf1-hold, 3,578 cells, of which the engine is bit-exact on 1,857.
- Also: `r3-bgrat/bgld.py` now runs its table only as a script, and takes `efn` and a doubles CF mode for reuse.
