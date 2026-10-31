---
id: T-3509
title: "Gamma-family held-out probe: a refreshable blind log-uniform sweep (4,100 cells) with 50-digit math and a per-region report (decision 0188 R4.2)"
milestone: M9
lane: research
status: in-progress
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
- [ ] Cases under eval/suites/probes-numeric/cases (the probe's prefix), about 4,100 cells, filled by
      `rustsheet-oracle fill -b excel-graph`, with `rustsheet-accuracy math` sidecars for every cell.
- [ ] The generator is committed with its seed and a refresh procedure; a new seed draws new points.
- [ ] The scoring script prints the per-region table (exact / within 4 ulp / within 15 digits / visibly off, against
      Excel and against the math) and its output for main is committed.
- [ ] A README says the probe is never fitted, that any fit change is scored on it before and after, and how to read
      the report.
- [ ] Tasks filed for the blind regions it shows.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any region (the filed tasks); the GAMMA.INV(0.5,7990464.8,1) `#NUM!` (fixed elsewhere).
