---
id: T-3600
title: "FORECAST.ETS: AAA exponential-smoothing fit and forecast adapted from gridcraft (owner ruling, unverified against Excel)"
milestone: M14
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3599]
---

## Goal
FORECAST.ETS returns the AAA model's forecast at `target_date`, adapted from gridcraft (initial states, the smoothing filter, the α/β/γ optimiser and the h-step forecast) on T-3599's series core, under decision 0221.

## Context
Owner request (2026-11-01, session https://claude.ai/code/session_0179K7WDtouX929jBuhJCWHr): after the gridcraft
head-to-head (docs/research/gridcraft-head-to-head-2026-10-31.md, branch `claude/wonderful-cray-9q8ljx`), implement
the FORECAST.ETS family taking inspiration from gridcraft's `crates/functions/src/ets.rs` (67e39fae; MIT OR Apache-2.0,
clean-room from Hyndman et al. 2008). Excel for the web never computes the family and no desktop value exists
(statistics.md "FORECAST.ETS family: excel-graph never computes it", T-1553), so the owner ruled to ship the model
unverified (decision 0221). The argument step evidenced on excel-graph (T-2424, `forecast_ets_lifted`) runs first and
does not change; T-2425 matches desktop values once they exist.

Adapt gridcraft's `initial_states`, `filter`, `optimise` (grid then Nelder–Mead in the unit box), `fit` and
`Fit::forecast` into `statistical/ets.rs`; `seasonality` is 1 (detect), 0 (none) or 2 to 8,760, else `#NUM!`; a
`target_date` before the last time stamp is `#NUM!` (gridcraft's rule; Microsoft's documentation says so too). Keep
the work bounded (directive 5): the optimiser's evaluations times the series length, and any lifted repeat (memoise
per series and season, as `EtsData` does for SEASONALITY).

## Acceptance criteria
- [x] FORECAST.ETS computes the forecast; exact data forecast exactly (a straight line continues; an exactly repeating
      season repeats), shown by unit tests.
- [x] statistics.md's model section gains the fit and forecast rules (UNVERIFIED), and any new cap a DEVIATION.
- [x] `make eval-baseline` shows no regression (or each one is named and accepted under decision 0221 by the owner).
- [x] `make check` green; reviewers approve.

## Out of scope
- CONFINT and STAT (T-3601).
- Matching desktop Excel (T-2425).

## Outcome
FORECAST.ETS computes the AAA model adapted from gridcraft (statistical/ets/model.rs: initial states, filter, grid
plus Nelder–Mead optimiser, forecast), unverified against Excel under decision 0221. `EtsData` memoises the series,
detected season and fit per series shape and season, and one call's detections and fits share a work budget of 2^30
(`MAX_CALL_WORK`, a DEVIATION), which also bounds the repeated SEASONALITY detections security noted on T-3599.
Unit tests: a straight line and an exactly repeating season with a trend forecast exactly; `target_date` before the
last point and an out-of-range `seasonality` are `#NUM!`. Eval: 169 cells fixed (gen-m9 168, probes 1), 0
regressions, all size-mismatch `#N/A` matches.
