---
id: T-2371
title: "Probe FORECAST.ETS family optional settings as array or range, and boolean, text and error target_date"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's shape for the FORECAST.ETS family when an optional setting is an array or range, and its result for a boolean, number-text or error `target_date` / `statistic_type`, is known, and ours matches.

## Context
T-2028 lifted `target_date` and `statistic_type` (oracle-backed) and also made the optional settings (seasonality, data_completion, aggregation, confidence_level) `Lift` per the catalog's `value` mode, unverified (docs/excel-semantics/statistics.md "FORECAST.ETS family"). The web gives `#N/A` for valid calls but `ROWS` shows the shape: probe `ROWS(FORECAST.ETS(9,A1:A8,E1:E8,{1;4}))`, a `G1:G2` setting, CONFINT's confidence_level, `"x"` as CONFINT target_date and STAT statistic_type, and TRUE, `"9"` and `#N/A` as target_date or statistic_type. Kinds: functions/lift/kinds/statistical.rs (`ETS_*`). Use the excel-probe skill.

## Acceptance criteria
- [ ] A probes case covers the shapes above with oracle values; the kinds match them or statistics.md records the findings.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
