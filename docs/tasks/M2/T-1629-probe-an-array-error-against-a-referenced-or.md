---
id: T-1629
title: "Probe an array error against a referenced or direct error in GEOMEAN, HARMEAN, SKEW, KURT, VAR, STDEV, AVERAGEA, MAXA and MINA"
milestone: M2
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for how an error inside an array ranks against a referenced or a direct error in GEOMEAN, HARMEAN, SKEW, KURT, VAR, STDEV, AVERAGEA, MAXA and MINA, with ours fixed if it differs.

## Context
T-1615 probed SUM, MIN, MAX, AVERAGE and MEDIAN (probes/aggregate-array-error-vs-ref-error: the leftmost of the referenced, array and LAMBDA errors wins, a direct error beats them all) and made the shared walker hold array errors for every `Ranked` policy, which also changed these functions: UNVERIFIED extrapolation (syntax.md "Aggregate error precedence"). Extend the probe's D1-D40 shapes to them; the A-variants have their own reading rules (`A_ARGS`).

## Acceptance criteria
- [ ] A probe with oracle values for the shapes of D1-D40 on each function; syntax.md updated; ours fixed if it differs
- [ ] `make check` green; reviewers approve.

## Out of scope
-
