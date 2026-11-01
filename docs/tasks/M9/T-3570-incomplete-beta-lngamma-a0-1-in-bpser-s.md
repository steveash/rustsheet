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
Done 2026-11-01 (research; negatives with counts, nothing solved for the engine).
- docs/research/t2464/r5-bpser-lg/RESULTS.md. Probes t3570-r5-bpser-lg (5,043 cells: 504 a0 in single-ulp steps at
  7 centers × 3 b0, 8 x each, with Excel's GAMMALN columns) and t3570-r5-bpser-lg-aux (2,520) were filled from
  excel-graph. The positive control recovers the planted g on 504 of 504 a0.
- lnΓ(a0) in bpser's L is still a negative:
  - GL is right at 0.15 and 0.3 (in the best set on 143 of 144 a0) and leads from 0.7 up.
  - Just above 0.5, fl(gamln1 - LN) is right more often: 19 a0 against 7.
  - Five other distinct forms lose (CR lnΓ(a0), fl(CR lnΓ(1 + a0) - LN a0) and three LN(GAMMA) forms; fl(GAMMALN(1 +
    a0) - LN a0) equals GL below 0.7 and the fl64 gamln1 form equals g1ln53), and so do the raw-t kernel and K3 kept
    past 1.5.
  - Overall GL wins 52 to 36 (tune) and 51 to 19 (hold), so the engine's GL53 stays.
- The ceiling also holds for a0 < 1 < b0 < 8: 3,355 of 4,032 cells, at 66-89%. It sits in alx, s or EXP, not in L.
  None of the series or composition variants tried lifts it.
- Confirmed T-1941's identity GAMMALN(v) = fl(GAMMALN(fl(1 + v)) - LN v) below 0.7 (360 of 360).
- No engine follow-up: nothing is solved. Research follow-up T-3771 (pin lnΓ on (0.5, 0.56]; LN and EXP side columns
  for the ceiling).
- Eval delta: none (research; probes-numeric is not gated, and no engine code changed).
