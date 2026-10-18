---
id: T-1636
title: "Probe the final division of AVERAGEA, AVERAGEIF and AVERAGEIFS for double rounding"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
AVERAGEA, AVERAGEIF, AVERAGEIFS, SUBTOTAL(1,101) and AGGREGATE(1) divide their sum by the count the way Excel does (twice-rounded, as AVERAGE), with oracle evidence.

## Context
T-1619 showed AVERAGE's final division rounds twice (probes-numeric/average-large-count-division, counts 2051-4095; no count <= 2048 discriminates). `fn_averagea` (functions/statistical/count_and_average.rs) and the AVERAGEIF(S) total/count (functions/statistical/if_ifs.rs) still use plain `sum / count`. Reuse the T-1619 generator idea: one 15-digit S and n-1 zeros (or matching criteria cells), n in 2051..4095, S picked by exact simulation. Fix with `arithmetic::extended_quotient` if Excel double-rounds.

## Acceptance criteria
- [ ] Probes-numeric cases with oracle values for each function; the engine matches, or the divisions are changed to `extended_quotient`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes-numeric/average-family-large-count-division` (36 cells, excel-graph): Excel double-rounds the final division in AVERAGEA, AVERAGEIF(S), SUBTOTAL(1/101); AGGREGATE(1) already matched via AVERAGE. Changed the three divisions to `extended_quotient`. numbers.md updated. Eval: 36 new probe cells (30 newly fixed, the 6 AGGREGATE(1) cells already passed), 0 regressions; probes-numeric is not a gated suite. Follow-up: T-1648 (DAVERAGE).
