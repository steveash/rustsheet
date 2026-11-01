---
id: T-3599
title: "FORECAST.ETS family core and FORECAST.ETS.SEASONALITY: adapt gridcraft's AAA series and season detection (owner ruling, unverified against Excel)"
milestone: M14
lane: functions-stat
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
FORECAST.ETS.SEASONALITY returns the season length of its series, and the series building every member shares (pairing, timeline step, aggregation, data completion, season detection) exists once, adapted from gridcraft with attribution, under decision 0221's owner ruling.

## Context
Owner request (2026-11-01, session https://claude.ai/code/session_0179K7WDtouX929jBuhJCWHr): after the gridcraft
head-to-head (docs/research/gridcraft-head-to-head-2026-10-31.md, branch `claude/wonderful-cray-9q8ljx`), implement
the FORECAST.ETS family taking inspiration from gridcraft's `crates/functions/src/ets.rs` (67e39fae; MIT OR Apache-2.0,
clean-room from Hyndman et al. 2008). Excel for the web never computes the family and no desktop value exists
(statistics.md "FORECAST.ETS family: excel-graph never computes it", T-1553), so the owner ruled to ship the model
unverified (decision 0221). The argument step evidenced on excel-graph (T-2424, `forecast_ets_lifted`) runs first and
does not change; T-2425 matches desktop values once they exist.

Code: new `crates/rustsheet-engine/src/functions/statistical/ets.rs` (the series core and season detection) and
`statistical/forecast.rs` (`forecast_ets_lifted` reads `values` and `timeline` once and hands each lifted element to
the member's computation).

## Acceptance criteria
- [x] Decision 0221 records the owner ruling, the licence and provenance check and the consequences; NOTICE and
      ets.rs keep gridcraft's MIT notice.
- [x] FORECAST.ETS.SEASONALITY computes the detected season (4 on the period-4 series of
      probes/forecast-ets-longer-series); FORECAST.ETS, CONFINT and STAT stay `#N/IMPL!` (T-3600, T-3601).
- [x] statistics.md has the UNVERIFIED model section and a DEVIATION section for the work caps (directive 5).
- [x] `make eval-baseline` shows no regression; unit tests cover the series errors, completion, aggregation,
      monthly timelines and season detection.
- [x] `make check` green; reviewers approve.

## Out of scope
- FORECAST.ETS's fit and forecast (T-3600); CONFINT and STAT (T-3601).
- Matching desktop Excel (T-2425, after T-1553).
