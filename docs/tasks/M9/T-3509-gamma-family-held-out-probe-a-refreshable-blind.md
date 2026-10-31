---
id: T-3509
title: "Gamma-family held-out probe: a refreshable blind log-uniform sweep (4,100 cells) with 50-digit math and a per-region report (decision 0188 R4.2)"
milestone: M9
lane: research
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The gamma family (GAMMA.DIST, CHISQ.DIST.RT, GAMMA.INV, CHISQ.INV.RT, POISSON.DIST, GAMMALN, GAMMA) has a permanent,
ungated, held-out probe in probes-numeric: a seeded log-uniform sweep over each function's full domain, oracle-filled
from Excel, with 50-digit math references attached as math sidecars, and a per-region report (function x shape bucket
x x/a or p bucket) against Excel and against the math. A refresh with a new seed draws new, never-fitted points, so the
probe stays a hold-out (decision 0188 R4.2).

## Context
- A blind sweep of 4,100 cells at main 3003a898b (excel-graph fill, 2026-10-31) found visibly-off regions no fit had
  measured: GAMMA.DIST cdf 44 and pdf 41 of 500 (lower tail, shape 2.5 to 20), CHISQ.DIST.RT 24 (far tail at a small
  df; shape 20 and above with x/a 1.4 to 10), GAMMA.INV 54 (shape below 0.5 with p in the upper half; deep lower tail
  at shape 0.5 to 20), POISSON.DIST cdf 41 (far tail, mean well above k) and pmf 15 (k above 1000, mean near k);
  CHISQ.INV.RT, GAMMALN and GAMMA 0. Every cell is within 4.6e-13 of the math, inside Gate A's 1e-9.
- Decision 0188 R4: fits are scored on a hold-out and on a fresh sweep; this probe is that fresh sweep for the
  gamma family, kept and refreshable.
- Gate A math for eval cells (T-3380) needs a definition per function in tools/src/rustsheet_tools/accuracy/defs/.

## Acceptance criteria
- [x] Cases under eval/suites/probes-numeric/cases (the probe's prefix), about 4,100 cells, filled by
      `rustsheet-oracle fill -b excel-graph`, with `rustsheet-accuracy math` sidecars for every cell.
- [x] The generator is committed with its seed and a refresh procedure; a new seed draws new points.
- [x] The scoring script prints the per-region table (exact / within 4 ulp / within 15 digits / visibly off, against
      Excel and against the math) and its output for main is committed.
- [x] A README says the probe is never fitted, that any fit change is scored on it before and after, and how to read
      the report.
- [x] Tasks filed for the blind regions it shows.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing any region (the filed tasks); the GAMMA.INV(0.5,7990464.8,1) `#NUM!` (fixed elsewhere).

## Outcome
- Probe: eval/suites/probes-numeric/cases/t3509-blind-{gamma-dist,chisq-dist-rt,gamma-inv,chisq-inv-rt,poisson-dist,
  gammaln-gamma} (4,100 cells, the 2026-10-31 sweep's points, seed 20261031), excel-graph refill identical to the
  sweep's fill at all 4,100 cells; math sidecars for all 4,100 (`rustsheet-accuracy math --filter`, new).
- Gate A definitions for the gamma family (tools/src/rustsheet_tools/accuracy/defs/gamma.py): the incomplete gamma by
  its series and Legendre's continued fraction (mpmath's `gammainc` gives up at shapes of 1E6 and up), quantiles by a
  safeguarded Newton on ln x.
- Report (docs/research/t3509/score-main-744815d06.out), ours vs Excel visibly off: GAMMA.DIST cdf 44, pdf 41,
  CHISQ.DIST.RT 24, GAMMA.INV 55 (one `#NUM!`, the only cell outside Gate A), CHISQ.INV.RT 0, POISSON cdf 40, pmf 14,
  GAMMALN 0, GAMMA 0. 227 of the 500 GAMMA.INV cells are p = 0.5 (a quirk of the first draw, kept; later seeds redraw).
- Tasks: T-3519 (GAMMA.INV median below shape 0.5), T-3520 (GAMMA.INV deep lower tail at shape 0.5-20 outside T-3311's
  model), T-3521 (CHISQ.DIST.RT at shape 20+, x/df 1.4-10), T-3522 (POISSON pmf at k > 1000 near the mean); evidence
  for GAMMA.DIST at shape 2.5-20 added to T-3219 (T-3412 lives on T-3311's unmerged branch, so it was not edited).
  Unowned and recorded in the README: GAMMA.DIST at shape 0.5-2.5 and 20-1E3 low x/a (37 cells), POISSON cdf near the
  mean (5).
- statistics.md: "Gamma family on a blind log-uniform sweep: where Excel is far from the math (T-3509)".
