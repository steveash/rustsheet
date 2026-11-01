---
id: T-3601
title: "FORECAST.ETS.CONFINT and FORECAST.ETS.STAT from the shared AAA fit (owner ruling, unverified against Excel)"
milestone: M14
lane: functions-stat
status: done
priority: P3
size: M
tier: opus
depends_on: [T-3600]
---

## Goal
FORECAST.ETS.CONFINT and FORECAST.ETS.STAT return the interval half-width and the fit statistics of T-3600's model, adapted from gridcraft, under decision 0221.

## Context
Owner request (2026-11-01, session https://claude.ai/code/session_0179K7WDtouX929jBuhJCWHr): after the gridcraft
head-to-head (docs/research/gridcraft-head-to-head-2026-10-31.md, branch `claude/wonderful-cray-9q8ljx`), implement
the FORECAST.ETS family taking inspiration from gridcraft's `crates/functions/src/ets.rs` (67e39fae; MIT OR Apache-2.0,
clean-room from Hyndman et al. 2008). Excel for the web never computes the family and no desktop value exists
(statistics.md "FORECAST.ETS family: excel-graph never computes it", T-1553), so the owner ruled to ship the model
unverified (decision 0221). The argument step evidenced on excel-graph (T-2424, `forecast_ets_lifted`) runs first and
does not change; T-2425 matches desktop values once they exist.

Adapt gridcraft's `Fit::forecast_sd` (the h-step variance, Hyndman et al. Table 6.1 class 1, with NORM.S.INV of
(1 + confidence_level) / 2; reuse RustSheet's normal quantile) and `forecast_ets_stat` (1 α, 2 β, 3 γ, 4 MASE, 5 SMAPE,
6 MAE, 7 RMSE, 8 step size); `confidence_level` outside (0, 1) and `statistic_type` outside 1 to 8 are `#NUM!`.

## Acceptance criteria
- [x] test_tailed_array.rs `fill` gives H4 a formula that is still `#N/IMPL!` (it uses FORECAST.ETS.STAT until this
      task; an error literal does not do, the tailed reading meets it as `#ERROR!`), and its tests pass.
- [x] CONFINT and STAT compute; unit tests show the values on exact data and each statistic type.
- [x] statistics.md's model section gains their rules (UNVERIFIED).
- [x] `make eval-baseline` shows no regression (or each one is named and accepted under decision 0221 by the owner).
- [x] `make check` green; reviewers approve.

## Out of scope
- Matching desktop Excel (T-2425).

## Outcome
FORECAST.ETS.CONFINT and FORECAST.ETS.STAT compute from T-3600's fit, unverified against Excel under decision 0221.
`Fit` now keeps its α, β and γ and its one-step errors, and gains `forecast_sd`, gridcraft's closed form of Hyndman et
al. Table 6.1, class 1. CONFINT reuses `normal::normal_quantile`. STAT computes statistics 1 to 8, and statistic 8 is
`Series::step`. `test_tailed_array`'s `#N/IMPL!` fixture moved to `SHEETS(5)`. T-3600's deferred review points are
also done here: unit tests of the model internals, the `MAX_CALL_WORK` doc rewrap, the stale "Ours" wording, the
quoted Microsoft sentence on `target_date`, and T-2425's probe list.
Eval: 318 cells fixed (gen-m9 317, probes 1), 0 regressions. All are size-mismatch `#N/A` matches.
