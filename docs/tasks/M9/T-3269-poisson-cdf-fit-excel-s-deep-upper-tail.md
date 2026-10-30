---
id: T-3269
title: "POISSON cdf: fit Excel's deep upper tail (mean well above k)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3289]
campaign: visible
---

## Goal
POISSON and POISSON.DIST's cdf match Excel in the deep upper tail, where the mean is well above k (Q(k + 1, mean) with mean >= about 4(k + 1)), without losing any Excel-computed cell of CHISQ.DIST.RT, CHIDIST or GAMMA.DIST that shares the kernel.

## Context
T-2055 fitted the pmf (docs/excel-semantics/statistics.md "POISSON.DIST pmf: two closed forms") and left the cdf on
`statistical/incomplete_gamma` `gamma_ur`. In the deep upper tail Excel is up to 34 ulp from the exact value and up to
60 from ours, with no shared pattern: `POISSON(2,100,TRUE)` = 1.8976107553682424E-40 (gen-m9 c7664e494e418395 A294; ours -35
ulp), `POISSON(3,100,TRUE)` (+60), `POISSON(6,100,TRUE)` (583ca2baea0bb30a A399, -34), `POISSON(9,100,TRUE)` (+44),
`POISSON(6,180,TRUE)` (+24), `POISSON(3,42,TRUE)` (+10). Not the pmf times its finite series 1 + k/m + ...: with Excel's own
pmf that is 45 ulp off at (2, 100). Not exp(fl(a LN x - x - GAMMALN(a))) times the series in any of 48 storage orders
tried (T-2055 scratch fit). This is the shared incomplete-gamma kernel's Q prefactor; coordinate with T-2638 and T-3060.

Stage (decision 0114): `campaign: visible`. The cdf evidence is mixed: away from the deep upper tail Excel is within
about 2 ulp of the exact value (the `accuracy` premise holds there, and the accurate kernel already matches), but in the
tail these cells are, Excel is the inaccurate side (4 to 34 ulp from the exact value at mean 100, k 1 to 5), so matching them
means reproducing Excel's own prefactor, not a more accurate kernel.

Gap triage 2026-10-30 (`make eval` on origin/main 73b187224; `scripts/exit_owners.py --all-off` listed these with no
open owner). The same deep-tail Q(a, x) shows in CHISQ.DIST.RT and CHIDIST, so this task now owns those cells too;
ulps of Excel's value against 60-digit mpmath, Excel / ours:
- POISSON: `POISSON(1,100,TRUE)` -12.1 / +0.9; `POISSON(3,42,TRUE)` -10.9 / -0.9; `POISSON(9,100,TRUE)` -43.3 / +0.7;
  `POISSON.DIST(3,100,TRUE)` (x = 3.14159 truncated) -59.4 / +0.6; `POISSON(8,42,TRUE)` +22.6 / +0.6;
  `POISSON(2,100,TRUE)` (873f30d2bf76f8db, k from ERROR.TYPE) +34.2 / -0.8; `POISSON(6,180,TRUE)` (mean from COUPDAYS)
  -25.3 / -1.3; and gen-m9/3c06f9298f65a3ba A91, a CUMIPMT whose rate is 1/`POISSON.DIST(10,42,TRUE)` (3.4E-15
  relative off; if the cdf fix does not move it, probe the sub-expression before claiming it).
- CHISQ.DIST.RT / CHIDIST: `(200,5)` -32.3 / -0.3; `(1000,5)` -43.1 / -3.1; `(200,20)` = `POISSON(9,100,TRUE)`
  -43.3 / +0.7; `(1000,3)` +86.7 / -1.3. Also off but owned elsewhere: `CHIDIST(1000,30)` +405 (m9-legacy-chidist A71,
  A72), and `CHIDIST(100,4)` -38.1 and `(100,5)` +33.8 (gen-m9 3a25a03eb94fcfbd A158, bbdfc06cae0e0d71 A125), which the
  gap-triage-2026-10-30d branch hands to T-3219; whichever task lands Excel's deep-tail prefactor first takes the others'
  deep-tail cells with it (record the split in both).
- Needs an oracle sweep: these points have "no shared pattern" (above) and 48 storage orders already failed, so the
  fit waits for T-3289's dense sweep (POISSON k 0 to 19 at mean 4(k + 1) to 100(k + 1), the CHISQ.DIST.RT twins,
  half-integer shapes and ulp runs).

Cells (the gen-m9 and probe cells above):
- gen-m9/097af1ca33638670.case.json (1): Calc!A214
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A214
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A80
- gen-m9/9735c5ab72fa5ecf.case.json (1): Calc!A48
- gen-m9/09b47629f54b9817.case.json (1): Calc!A349
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A124
- gen-m9/d208bbafc3bc1997.case.json (1): Calc!A169
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A91
- probes-numeric/m9-legacy-chidist.case.json (4): Calc!A61, A62, A69, A70
- probes-numeric/t1049-chisqdistrt.case.json (3): Calc!A27, A28, A39

## Acceptance criteria
- [ ] The POISSON, CHISQ.DIST.RT and CHIDIST cells above are within 15 digits of Excel, and no Excel-computed cell of POISSON, POISSON.DIST,
      CHISQ.DIST.RT, CHIDIST, GAMMA.DIST or GAMMADIST goes from pass to fail (main-vs-HEAD per cell).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The pmf (T-3247).
