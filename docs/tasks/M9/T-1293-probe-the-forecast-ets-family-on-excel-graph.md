---
id: T-1293
title: "Probe the FORECAST.ETS family on excel-graph: when is it a number, not #N/A"
milestone: M9
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Evidence for when FORECAST.ETS, FORECAST.ETS.CONFINT, FORECAST.ETS.SEASONALITY and FORECAST.ETS.STAT return a number on excel-graph, recorded in docs/excel-semantics/statistics.md.

## Context
probes/stat-argument-modes (excel-graph, 2026-10-09, T-1080; legend in docs/excel-semantics/statistics.md "Statistical function argument modes"): all four are `#N/A` on excel-graph for an 8-point series on the timeline 1..8 (as ranges and as array literals),
yet lift a scalar argument (ROWS 2). Is ETS unavailable in Excel for the web, or does it need more points or a seasonality
argument? Probe longer series (e.g. 24 and 36 points, seasonality 0/1/4) before anyone implements the family (the engine
is `#N/IMPL!`). If excel-graph never computes it, mark the catalog entries `oracle: none` per decision 0039 / M14.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/1782efcecd5d4b1b Calc!A27 `=FORECAST.ETS.CONFINT(..,Data!A8:C8,Data!A11:C11,ROUNDDOWN((7/Data!A4),0.1))` (timeline " x ", FALSE, 123456789.123): Excel `#VALUE!`, engine `#N/IMPL!`.
- gen-m9/3462856259a37891 Calc!A28 `=FORECAST.ETS.CONFINT(0.25,(SUMIFS(..)-RADIANS(..)),MINA(Data!B5:B12),NA())` (values `#VALUE!`): Excel `#VALUE!`, engine `#N/IMPL!`.
- gen-m9/55fc8c4ffbd7c7cc Calc!A8 `=HYPERLINK(..,(FORECAST.ETS.CONFINT(Data!D5,Data!A11:D12,Data!A4:B12)+MINVERSE(Data!B4:C4)))` (8 values, 18 timeline cells): Excel `#N/A`, engine `#N/IMPL!`.
These are the only ETS calls in gen-m9 and all three are argument errors, so they score argument validation, not the algorithm (statistics.md "FORECAST.ETS.CONFINT in the generated gen-m9 suite: argument errors only"). The probe should also say whether a validation-only implementation (errors, then `#N/A` like excel-graph) is acceptable or whether the family is `oracle: none`.

## Acceptance criteria
- [ ] A probe case with excel-graph values answers the question; the finding is in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
