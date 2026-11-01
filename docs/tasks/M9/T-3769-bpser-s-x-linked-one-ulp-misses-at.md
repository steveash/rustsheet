---
id: T-3769
title: "bpser's x-linked one-ulp misses at 1 < b0 < 8 (a0 near 0.3) and the a0-linked residual at 0.5 and 0.7 (T-3570)"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
The stored intermediate behind bpser's x-linked one-ulp misses is identified (or the candidates tried are recorded with
their counts), and the a0-linked residual at a0 near 0.5 and 0.7 is explained or narrowed.

## Context
- Split from T-3570 (docs/research/t2464/r5-bpser-lngamma/RESULTS.md). T-3570 settled lnGamma(a0 < 1) inside bpser's L
  as Excel's GAMMALN (GL53 beats TOMS's gamln1 form in all 12 runs, tune and hold-out). It left two residuals on
  probes-numeric t3570-r5-bpser-lngamma-beta (b0 2.5 and 6.25, a0 by single ulps around 0.3, 0.5, 0.7, two x each):
  - **x-linked**: at a0 near 0.3 the misses do not follow a0 (both x miss together at chance rate; at b0 6.25 every
    miss is at x 0.0371, none at 0.0213), and every one is the model 1 ulp above Excel (`residual.out`). This looks
    like r2-bpser-small's "b < 1 ceiling" (one stored intermediate, s or E, beating with x; `misspos_*.out`,
    `fixwhich.out`), now seen at 1 < b0 < 8. That item of T-3570 was not attempted, so it moves here.
  - **a0-linked**: at 0.5 and 0.7 both x miss together 2-3 times more often than chance. No constant L shift in
    -2..2 ulps fits: 155 of 960 a0 fit only at some k != 0 (mostly +1, +2) and 169 at none (`lshift.out`).
    Candidates: an internal lnGamma(a0) at a different precision than the GAMMALN worksheet function, or alx's rounding.
- Method: docs/research/excel-kernel-playbook.md; work in a new docs/research/t2464/r6-* directory and import
  r5-bpser-lngamma's and r2-bpser-small's scripts instead of copying them. Rules: positive controls first, a real
  hold-out, record negatives (decision 0056).
- Suggested probe: x runs of consecutive doubles at fixed (a0, b0) = (0.3, 6.25) and (0.3, 2.5), so L is constant and
  the r2 `lcover` inversion applies; separate s from E by also probing EXP at the z values the model computes.

## Acceptance criteria
- [ ] A RESULTS.md records a model with exact, within-4-ulp and within-15-digit counts on a held-out probe, or the
      negatives tried with their counts, for both residuals.
- [ ] New oracle probes are committed under eval/suites/probes-numeric/cases/ with their sidecars.
- [ ] statistics.md's T-2464 bpser entry is updated with what is found.
- [ ] Engine follow-ups are filed for any solved piece.
- [ ] `make check` green; reviewers approve.

## Out of scope
- b0 >= 8 algdiv (T-3569), engine GAMMALN accuracy (T-3042), engine changes.
