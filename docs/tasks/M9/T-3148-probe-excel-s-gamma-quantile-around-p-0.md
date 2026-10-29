---
id: T-3148
title: "Probe Excel's gamma quantile around p = 0.999: GAMMA.INV, CHISQ.INV and CHISQ.INV.RT at p near 0.999 and the cdf at the doubles beside Excel's quantile"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An oracle-filled probe case gives the Excel points T-3146 needs to fit where Excel's gamma quantile lands inside the
band of doubles whose double-valued cdf rounds to p: quantiles at several p near 1, and Excel's own cdf at the doubles
around each quantile it returns.

## Context
Near p = 0.999 Excel's GAMMA.INV, GAMMAINV, CHISQ.INV and CHISQ.INV.RT are 18 to 106 ulp from the true quantile (a
50-digit mpmath bisection, gap triage 2026-10-29), in either direction, while the engine is within an ulp of it
(docs/excel-semantics/statistics.md, the "Excel's gamma quantiles (T-0972 ...)" bullet). The cached probes have
only one p between 0.95 and 1 - 1E-10 (0.999), and no Excel cdf at the doubles beside Excel's quantile, so the
landing rule (Excel's starting point, step and stopping test) cannot be fitted from eval/ alone. At p = 1 - 1E-10 the
same probes are within 2 ulp, so whatever Excel does changes between those two p. Probe design follows the
`excel-probe` skill; formulas are literal calls (no cell references) so `rustsheet-fitlab` can read the arguments.

## Acceptance criteria
- [ ] A spec case `probes-numeric/t3145-gamma-quantile-flat-band`, oracle-filled (excel-graph), no pending cells,
      about 350 literal cells:
  - `GAMMA.INV(p,a,1)` for p in 0.99, 0.995, 0.998, 0.999, 0.9995, 0.9999, 0.99999 and a in 0.5, 1, 1.5, 2, 2.5,
    5, 9, 25, 50;
  - `CHISQ.INV.RT(p,df)` for the same p and df in 1, 2, 3, 5, 10, 20, 50, 100;
  - `GAMMA.INV(p,2,1)`, `GAMMA.INV(p,5,1)` and `CHISQ.INV.RT(p,5)` at the three doubles each side of 0.999
    (17-digit literals);
  - Excel's cdf around each of its p = 0.999 quantiles that T-3146 owns: `GAMMA.DIST(x,a,1,TRUE)` (and
    `CHISQ.DIST(x,df,TRUE)`, `CHISQ.DIST.RT(x,df)` for the chi-square cells) at x = Excel's quantile moved by 0, ±1,
    ±2, ±4, ±8, ±16, ±32, ±64 and ±128 ulp (17-digit literals).
- [ ] docs/excel-semantics/statistics.md names the case under the gamma-quantile bullet and states, from the filled
      values, at which p Excel leaves the true quantile by more than 4 ulp and whether its quantile sits at an edge
      of the band of x whose Excel cdf equals p.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing the landing rule (T-3146).
- The deep tails (T-2561) and shapes above 1E9 (T-3070).
