---
id: T-3688
title: "Probe y-only and near-DBL_MAX overflow in STEYX, INTERCEPT, FORECAST and the correlation family"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
STEYX, INTERCEPT, FORECAST and the correlation family match Excel when only y's sum of squares overflows, when Σx overflows (data near DBL_MAX) and when only the sum of products overflows.

## Context
T-2080 (probes/correl-inexact-constant-overflow, statistics.md "Paired regression functions") made overflowing Σ(x-x̄)² and Σ(y-ȳ)² `#NUM!` in CORREL, PEARSON, RSQ and, for x's sum, SLOPE, INTERCEPT, STEYX, FORECAST. Unprobed there: STEYX, INTERCEPT and FORECAST with only y overflowing (ours: STEYX `#DIV/0!` because the variance is inf; INTERCEPT and FORECAST finite), x near DBL_MAX where Σx itself overflows and the mean is inf (ours `#NUM!`), and a sum of products that overflows while both sums of squares are finite (ours `#DIV/0!`). Code: `PairedMoments` in crates/rustsheet-engine/src/functions/statistical/correl.rs. Use the excel-probe skill; fix any mismatch if it is a small change in `PairedMoments`, else file a task.

## Acceptance criteria
- [ ] A filled probe case covers the three inputs above for STEYX, INTERCEPT, FORECAST, CORREL and SLOPE; the engine matches every cell, or the entry records what stays open.
- [ ] Findings recorded in statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
