---
id: T-3570
title: "Incomplete beta: lnGamma(a0 < 1) in bpser's L and the b < 1 bpser ceiling"
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
bpser's L is modelled where it takes lnGamma(a0 < 1), and the stored intermediate behind the b < 1 bpser ceiling is found.

## Context
- Split from T-3468 (size; one task per open item), itself split from T-3383 and T-2512. T-3383 pinned the density's
  lnB at min < 1 < 8 < max on 156 of 160 shapes (docs/research/t2464/r4-algdiv/RESULTS.md). Work in
  docs/research/t2464/ in a new `r5-*` directory, importing r2-bratio's `bratio_excel.py`, r2-tdist's `xlk.py`,
  r3-bgrat's `data.py`/`bscore.py` and r4-algdiv's `scan.py`. Method: docs/research/excel-kernel-playbook.md.
- Rules: positive controls before trusting a method; a real hold-out; record negatives (decision 0056); truncate
  formula literals to 15 digits as Excel does.
- **lnGamma(a0 < 1) inside bpser's L** depends on the shape: GL below 0.3, fl(gamln1(v) - LN v) at w2's (0.5, 6.25). On
  the old 1 < b0 < 8 cells GL53 gets 3,365 against r2-bratio's 4,038 of 7,455. Probe bpser at fixed b0 with a0
  stepping by single ulps around 0.3, 0.5 and 0.7.
- **The b < 1 bpser ceiling.** At any constant L, b < 1 shapes top out at 55-84% bit-exact. The misses come in
  sawtooth blocks, each fixed by moving s or E one ulp, so the cause is one stored intermediate (r2-bpser-small
  `misspos_*.out`, `fixwhich.out`).

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
- **lnGamma(a0 < 1) inside bpser's L is Excel's GAMMALN** (docs/research/t2464/r5-bpser-lngamma/RESULTS.md). New probe
  probes-numeric t3570-r5-bpser-lngamma-beta (1,920 BETA.DIST cells): b0 2.5 and 6.25, a0 stepping by single ulps around
  0.3, 0.5 and 0.7, two x each. Its -gl companion has Excel's GAMMALN at all 620 shape arguments. Both were filled from
  excel-graph with 0 failures.
  - GL53 beats TOMS's fl(gamln1(a0) - LN a0) in all 12 runs: 672 against 573 of 960 bit-exact on the tune half (b0 2.5)
    and 831 against 691 on the hold-out (b0 6.25). On the 457 cells where the forms differ, GL53 is right on 331 and
    gamln1 on 92.
  - The engine already uses this L (`bratio/bpser.rs`, T-2505), so no engine task follows. Its GAMMALN accuracy below 1
    stays with T-3042.
- **Not settled, moved to T-3769** (opus, research):
  - The b < 1 ceiling item was not attempted.
  - GL53's remaining 417 misses (all within 2 ulp): at a0 near 0.3 they follow x (model 1 ulp high). At 0.5 and 0.7 they
    partly follow a0, and no constant L shift of -2..2 ulps fits (`residual.out`, `lshift.out`).
- statistics.md's T-2464 bpser entry now states the GL(a0) result and points the open items at T-3769.
- **Eval delta**: probes-numeric gains 2,540 cells: 1,346 / 1,920 pass in -beta and 495 / 620 in -gl. Every failing cell
  is NumericUlp or NumericClose (within 15 digits), so none is visibly off. `make exit-owners MILESTONE=M9` lists no
  t3570 cell. The gated suites and their baselines are unchanged.
- No fitted value: the comparison is between two fixed formulas, so decision 0188 R4 has no `FITTED` range to record.
  The tune/hold-out split by b0 was fixed before scoring.
