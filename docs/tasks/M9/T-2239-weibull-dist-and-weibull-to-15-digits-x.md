---
id: T-2239
title: "WEIBULL.DIST and WEIBULL to 15 digits: (x/lambda)^k through exp(k*ln(x/lambda)), as Excel's tail values show (18 visibly-off sweep cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
WEIBULL.DIST and WEIBULL agree with Excel to 15 significant digits on the in-domain cells (x > 0)
of the three Weibull sweeps. Today 18 of them are visibly off. Excel's tail densities are less
accurate than the true value, so this means matching Excel's evaluation, not computing a more
accurate value.

## Context
Gap triage 2026-10-21 (`make eval` on `main` d77d01b0). `functions/statistical/weibull.rs`
`weibull_dist_of` calls `statrs::distribution::Weibull` (statrs 0.18: `cdf` = `-expm1(-x.powf(k) * l.powf(-k))`,
`pdf` = `k/l (x/l)^(k-1) exp(-x^k l^-k)`). T-0978 owns the last digits: it targets the same sweeps
bit for bit but has no 15-digit (visible-parity) stage, so the visibly-off sweep cells had no
visible-parity task (decision 0083).

The visibly-off cells (excel-graph; probes-numeric is ungated, so read them from `make eval` /
`make eval-show CASE=probes-numeric/<case>.case.json`):

| case / cell | formula | Excel | ours |
|---|---|---|---|
| m9-weibull-dist A22 | `WEIBULL.DIST(1000,0.5,1,FALSE)` | 2.9199312344877555e-16 | 2.9199312344877447e-16 |
| m9-weibull-dist A58 | `WEIBULL.DIST(5,1,1,FALSE)` | 0.006737946999085473 | 0.006737946999085467 |
| m9-weibull-dist A60 | `WEIBULL.DIST(10,1,1,FALSE)` | 4.539992976248477e-05 | 4.5399929762484854e-05 |
| m9-weibull-dist A62 | `WEIBULL.DIST(30,1,1,FALSE)` | 9.357622968840141e-14 | 9.357622968840175e-14 |
| m9-weibull-dist A64 | `WEIBULL.DIST(100,1,1,FALSE)` | 3.7200759760206773e-44 | 3.720075976020836e-44 |
| m9-weibull-dist A88 | `WEIBULL.DIST(1000,1,9,FALSE)` | 6.177532755504216e-50 | 6.177532755504306e-50 |
| m9-weibull-dist A104 | `WEIBULL.DIST(10,2,1,FALSE)` | 7.440151952041357e-43 | 7.440151952041672e-43 |
| m9-weibull-dist A124 | `WEIBULL.DIST(5,2,9,FALSE)` | 0.09067205826292972 | 0.09067205826292979 |
| m9-weibull-dist A130 | `WEIBULL.DIST(100,2,9,FALSE)` | 5.969559326940941e-54 | 5.969559326940687e-54 |
| m9-weibull-dist A144 | `WEIBULL.DIST(2,3.7,1,FALSE)` | 5.4560076923725256e-05 | 5.4560076923725154e-05 |
| m9-weibull-dist A146 | `WEIBULL.DIST(5,3.7,1,FALSE)` | 9.363099887555361e-166 | 9.363099887553769e-166 |
| m9-weibull-dist A159 | `WEIBULL.DIST(0.1,3.7,9,TRUE)` | 5.878989740656108e-08 | 5.8789897406560994e-08 |
| m9-weibull-dist A168 | `WEIBULL.DIST(5,3.7,9,FALSE)` | 0.07505415050895907 | 0.07505415050895915 |
| m9-weibull-dist A172 | `WEIBULL.DIST(30,3.7,9,FALSE)` | 4.605259708338221e-37 | 4.6052597083384175e-37 |
| t1048-weibull A8, B8 | `WEIBULL(10,3,8,FALSE)`, `WEIBULL.DIST(..)` | 0.08310360884023987 | 0.08310360884023976 |
| t1049-weibulldist A20 | `WEIBULL.DIST(10,1,1,FALSE)` | 4.539992976248477e-05 | 4.5399929762484854e-05 |
| t1049-weibulldist A22 | `WEIBULL.DIST(10,2,3,FALSE)` | 3.321186338840311e-05 | 3.321186338840322e-05 |

Lead, recorded in docs/excel-semantics/statistics.md "The far-tail density is less accurate than
exp(-t)". Compute t = (x/l)^k as exp(k*ln(x/l)). In a libm re-run over all 230 in-domain cells (x, k, l > 0 with a literal TRUE/FALSE cumulative; 116 cdf, 114 density):
- the cdf `-expm1(-t)` leaves no cell visibly off, and 94 of 116 are bit-exact, against 1 visibly off and 76 bit-exact for the
  engine's statrs form;
- the density `k/l*exp((k-1)*ln(x/l))*exp(-t)` leaves 4 visibly off (m9 A158, A168; t1048 A8, B8);
- `k/l^k*exp((k-1)*ln x)*exp(-t)` leaves 2 (t1048 A8, B8), against 17 for the engine's statrs form (17 + the cdf's 1 are the
  18 cells in the table).
- The same t also gives ironcalc-calc WEIBULL_DIST.xlsx E2 bit for bit (T-0978's line).

The fit is numeric, but the evidence is already in `eval/` (decision 0083 allows fitting at this stage).
Use the engine's own `ln`/`exp` kernels (numbers.md; fit lab, T-1523) before settling a form.
Extra candidate densities are cheap to test.

## Acceptance criteria
- [ ] Every cell in the table is within half a unit of the 15th significant digit of Excel's value.
- [ ] No other WEIBULL.DIST or WEIBULL cell in m9-weibull-dist, t1048-weibull, t1049-weibulldist or
      ironcalc-calc WEIBULL_DIST.xlsx becomes visibly off. A cell that moves from bit-exact to
      4 ulp or less follows decision 0083's allowance and is listed in the Outcome.
- [ ] eval/baseline/ironcalc-calc.txt gains no line. If WEIBULL_DIST.xlsx E2 or E3 leaves it, the
      baseline is updated with `make eval-baseline` and T-0978 is told in the Outcome.
- [ ] The chosen form is recorded in statistics.md (the entry above moves from lead to rule, with
      the cells it matches).
- [ ] `make check` green; reviewers approve.

## Out of scope
- x = 0 densities and boolean arguments (T-2035: t1048-weibull A16/B16, t1049-weibulldist A27).
- Bit-exact parity on the whole sweep (T-0978, after this stage).

## Visible-parity stage
This whole task is the visible-parity stage (decision 0083). Its target is every cell it owns within
half a unit of the 15th significant digit of Excel's value. Any bit-level miss left after that
stays with T-0978.
