---
id: T-2438
title: "Generator edge pools for M9 integer counts, FORECAST.ETS options and a sorted timeline"
milestone: M9
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
The M9 statistical params T-2255's sweep found without an edge pool draw from one, so the next gen-m9 sweeps (T-2256,
and a re-run of part 1) reach the inputs where Excel's checks change: integer counts and the FORECAST.ETS options.

## Context
T-2255 (gen-m9 statistical sweep A-L, 2026-10-22) listed these number parameters with no `pool` in
eval/catalog/functions.yaml; several of its new families (T-2429 HYPGEOMDIST bounds, T-2435 extreme arguments) sit
exactly on these edges and were hit only by chance:
- integer counts: BINOM.DIST, BINOMDIST and BINOM.DIST.RANGE `number_s`, `trials`, `number_s2`; BINOM.INV and
  CRITBINOM `trials`; the four HYPGEOM.DIST/HYPGEOMDIST counts; CONFIDENCE, CONFIDENCE.NORM and CONFIDENCE.T `size`;
  LARGE `k`. Edges: 0, 1, 2, a fraction that truncates (2.9), a negative fraction (-0.5), 2^31, 2^32, 1E15, text.
- FORECAST.ETS family options: `target_date`, `seasonality` (0-4 and 5, -1), `data_completion` (0, 1, 2),
  `aggregation` (1-7, 0, 8), `confidence_level` (0, 0.95, 1), `statistic_type` (1-8, 0, 9); and a `timeline`
  data pool that is long (24+ points), sorted and evenly spaced, so valid calls reach the algorithm (excel-graph gives
  `#N/A` for them, T-1293; desktop values are T-1553).
- BETADIST/BETAINV `A`/`B`, FISHERINV `y`, FORECAST/FORECAST.LINEAR `x`, and the lognormal `mean`
  (LOGNORM.DIST, LOGNORM.INV, LOGINV, LOGNORMDIST).
Template: T-2247 (tools/src/rustsheet_tools/gen/pools.py, the `pool:` keys in functions.yaml). Source each edge from
docs/excel-semantics/statistics.md, citing the entry in a pools.py comment as T-2247 did.

## Acceptance criteria
- [x] Every parameter listed above has a `pool:` in eval/catalog/functions.yaml backed by a pools.py pool with cited edges.
- [x] tools tests cover the new pools (ruff, mypy strict pass).
- [x] `make check` green; reviewers approve.

## Out of scope
- Running a sweep with them (T-2256 and later sweeps do that).

## Outcome
Added pools `count`, `ets_target`, `ets_seasonality`, `ets_completion`, `ets_aggregation`, `ets_confidence`, `ets_statistic`,
`ets_timeline`, `ets_values`, `beta_bound`, `fisherinv_y`, `forecast_x` and `lognorm_mean` (tools/src/rustsheet_tools/gen/pools.py,
edges cited from statistics.md) and put 58 listed params on them in eval/catalog/functions.yaml; test in tools/tests/test_generator.py.
Generator-only change: no eval delta. The ETS timeline and values pools are array constants whose first entries (24 points each) match in
length; they are drawn independently, so matching pairs come up by chance (correlated draws would need a generator change).
