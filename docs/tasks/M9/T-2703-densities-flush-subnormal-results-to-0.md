---
id: T-2703
title: "BINOM.DIST (k > 0), NEGBINOM.DIST, BETA.DIST, GAMMA.DIST, EXPON.DIST and WEIBULL densities flush a subnormal result to 0, as Excel does"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The densities Excel flushes return 0 instead of a subnormal: BINOM.DIST pmf off its k = 0 path, NEGBINOM.DIST, BETA.DIST
and GAMMA.DIST densities, EXPON.DIST and WEIBULL. That matches Excel, whose `(x)=0` companion of each of these cells is TRUE.

## Context
Found by T-2693 (excel-graph, 2026-10-25). Each cell's companion `=(<formula>)=0` is TRUE in Excel, so Excel returns a
true 0 where ours keeps a nonzero subnormal (e.g. `BINOM.DIST(4000000,10000000,p,FALSE)`, ours about 1.2E-320). Excel
keeps a subnormal from EXP, LOGINV and `BINOM.DIST(0,n,p,FALSE)` (the `(1-p)^n` path), whose companions are FALSE, so the
flush belongs to these functions, not to a global rule. docs/excel-semantics/numbers.md "Subnormal results become 0"
lists both sets. Owned cells, each target cell and its companion (both fail today):
- probes-numeric/t2513-r3-binom-pmf-n (64 targets and their companions): companions E1729, E1730, E1731, E1732, E1733, E1734 ...; e.g. `BINOM.DIST(4000000,10000000,(108963955*2^-28+62952151*2^-54),FALSE)`
- probes-numeric/t2513-verify-r3-binom-pmf-h (2 targets and their companions): companions J229, J238; e.g. `NEGBINOM.DIST(999,1,(70065857*2^-27+4559162*2^-53),FALSE)`
- probes-numeric/t2513-r3-m1-residual-hold (3 targets and their companions): companions E863, E875, E957; e.g. `BETA.DIST(A863,B863,C863,FALSE)`
- probes-numeric/m9-gamma-dist-a500 (1 targets and their companions): companions B22; e.g. `GAMMA.DIST(50000000,500,1000000,FALSE)`
- probes/t2693-subnormal-expon-dist A1, A2 (gen-m9/165db58d1540f014 Calc!A65 EXPON.DIST) and probes/t2693-subnormal-weibull A1, A2 (gen-m9/eccd03df42e40006 Calc!A96 WEIBULL); the two gen-m9 cells themselves too.
Code: the density paths in functions/statistical/ (binom.rs, beta.rs, gamma.rs, exponential.rs, and weibull's module).
Flush where the evidence says, a result with `0 < |x| < f64::MIN_POSITIVE` becoming 0. Keep the k = 0 BINOM.DIST path and
everything else as it is. Probe first if one function's flush point is unclear (for example whether BINOM.DIST flushes
for every k > 0 or only on one of its internal paths).

## Acceptance criteria
- [x] Every cell above passes; no baseline gains a line.
- [x] numbers.md records the flush per function, with these cells cited.
- [x] `make check` green; reviewers approve.

## Out of scope
- EXP, LOGINV and BINOM.DIST k = 0, which keep a subnormal in Excel and already pass.

## Outcome
Done. The EXPON.DIST, WEIBULL, BETA.DIST and GAMMA.DIST densities, the BINOM.DIST pmf off its k = 0 and k = n closed
forms and the NEGBINOM.DIST pmf now pass their result through `arithmetic::flush_subnormal`. Every owned cell and its
companion passes: t2513-r3-binom-pmf-n B/E1729:1792, t2513-verify-r3-binom-pmf-h B/J229, B/J238 (A/I229, A/I238 still
pass), t2513-r3-m1-residual-hold D/E863, D/E875, D/E957, m9-gamma-dist-a500 A22/B22, probes/t2693-subnormal-expon-dist
and -weibull A1:A2. `make eval-baseline`: gen-m9 fixed 2 (the two source cells), probes fixed 4, no regressions.
The closed forms and the cumulative modes are unchanged (no evidence that they flush).
