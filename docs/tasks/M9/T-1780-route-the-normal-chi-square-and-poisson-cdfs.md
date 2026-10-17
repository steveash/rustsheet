---
id: T-1780
title: "Route the normal, chi-square and Poisson CDFs through Excel's shared kernels"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1778]
---

## Goal
The engine computes the distribution functions that Excel builds from one another through the same compositions. Each then inherits any fix to the shared kernel, and the identities Excel shows hold bit for bit in RustSheet too.

## Context
T-1778 established bit-exact identities from oracle probes (docs/excel-semantics/statistics.md, "One incomplete-gamma kernel under ERF, ERFC, NORM, CHISQ, GAMMA.DIST and POISSON"; probes-numeric `kmyst-norm-erfc-a/b`, `kmyst-erf-gamma`, `kmyst-kernel-map`):
- `NORM.S.DIST(x,TRUE)` = `ERFC.PRECISE(-x*SQRT(0.5))/2`: multiply by the double sqrt(0.5), not divide by sqrt(2). `GAUSS(x)` = `NORM.S.DIST(x,TRUE)-0.5`.
- `NORM.DIST(x,m,s,TRUE)` = `NORM.S.DIST((x-m)/s,TRUE)`; `LOGNORM.DIST(x,m,s,TRUE)` = `NORM.DIST(LN(x),m,s,TRUE)`.
- `ERF.PRECISE(x)` = `GAMMA.DIST(x^2,0.5,1,TRUE)` = `CHISQ.DIST(2x^2,1,TRUE)`; `ERFC.PRECISE(x)` = `CHISQ.DIST.RT(2x^2,1)`.
- `CHISQ.DIST(x,k,TRUE)` = `GAMMA.DIST(x/2,k/2,1,TRUE)`; `GAMMA.DIST(x,a,b,TRUE)` = `GAMMA.DIST(x/b,a,1,TRUE)`; `POISSON.DIST(k,m,TRUE)` = `CHISQ.DIST.RT(2m,2(k+1))`.
- `T.DIST(x,n,TRUE)` for x < 0 = `T.DIST.2T(-x,n)/2`.
Today `special.rs` `normal_cdf` computes `0.5 * erfc((mean - x) / (std_dev * SQRT_2))`. That is a different rounding order from Excel's `fl(fl((x-m)/s) * fl(sqrt 0.5))` negated, then ERFC, then halved. Code: crates/rustsheet-engine/src/functions/statistical/{special,normal,log_normal,gauss,chisq,poisson,gamma*,t_dist*}.rs.

## Acceptance criteria
- [ ] Each identity above holds bit for bit in the engine, with a unit test per identity on the probe inputs.
- [ ] `make eval`: known failures do not grow in any suite. NORM_DIST.xlsx, LOGNORM.xlsx, GAUSS.xlsx, ERFs.xlsx, CHISQ/GAMMA/POISSON workbooks and the kmyst probes are compared before and after. Baseline shrinks are recorded with `make eval-baseline`.
- [ ] Fallback: if routing one function makes a suite regress, that function keeps its current path, and the task's Outcome records why.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cracking the kernel's own last digits (T-0968, T-1187, T-0971, T-0972).
