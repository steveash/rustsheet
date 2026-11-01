---
id: T-3641
title: "Re-probe on excel-graph whether the FORECAST.ETS family computes a number today (unblock check for T-2425)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Know whether Excel for the web (excel-graph) now computes the FORECAST.ETS family (FORECAST.ETS, FORECAST.ETS.CONFINT, FORECAST.ETS.SEASONALITY, FORECAST.ETS.STAT) on valid calls, or still gives `#N/A`, so T-2425 is either unblocked on excel-graph evidence or stays blocked on the `excel-desktop` backend for a recorded reason.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md, branch `claude/sharp-mendel-13x2km`). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The ETS family is still the engine's `#N/IMPL!` on every call that passes argument checks (T-2424), and is the largest
block of failing cells in the holdout themes note. T-2425 (implement) and T-1553 (desktop probe) are blocked on the
`excel-desktop` backend, which `uv run rustsheet-oracle status` still reports as `n/a` on 2026-11-01. The block rests
on T-1293's finding that excel-graph never computes the family (statistics.md "FORECAST.ETS family: excel-graph never
computes it"), which is weeks old; Excel for the web ships function updates without notice. Re-run
probes/forecast-ets-longer-series (and a fresh copy with seasonal, monthly and daily timelines) with `rustsheet-oracle
fill --refresh`, without touching the cached evidence of the old run.

## Acceptance criteria
- [ ] A fresh excel-graph run of probes/forecast-ets-longer-series, or a new case beside it, dated in the case, shows whether any valid call of each of the four functions now returns a number.
- [ ] If one does: statistics.md is updated, T-2425's `## Blocked because` is rewritten to need only excel-graph values (status `todo`), and a spec task for the AAA algorithm is filed (docs/specs/, decision on evidence source).
- [ ] If none does: statistics.md records the re-check date and T-2425 stays blocked; T-2371 is unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing any part of the ETS algorithm (T-2425 and its spec).
- T-2371's optional-settings probe.
