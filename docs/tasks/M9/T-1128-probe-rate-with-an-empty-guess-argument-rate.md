---
id: T-1128
title: "Probe RATE with an empty guess argument (RATE(48,-200,8000,0,1,))"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`RATE(48,-200,8000,0,1,)` (empty sixth argument) matches Excel, whether Excel takes the default guess 0.1 or 0.

## Context
T-1050 let `fn_rate` accept a sixth argument. An empty guess likely goes through `get_number` as 0 rather than the 0.1 default (functions/financial/mod.rs, guess branch); no oracle data covers it (found by the T-1050 fidelity review). Probe it (excel-probe skill), record the finding in `docs/excel-semantics/financial.md`, and fix if it differs.

## Acceptance criteria
- [x] A probe case with Excel values for an empty guess (and an empty fv/type for contrast) is recorded with its oracle values; cells that depend on solver convergence are baselined as known failures for T-0916.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/rate-empty-arguments` (9 cells, excel-graph): an empty guess behaves as guess 0 (not 0.1), which the engine already does (A1 and A3 agree). Findings in docs/excel-semantics/financial.md. No code change. The 8 cells that differ are all guess-0 convergence or last-digit differences, left to T-0916; eval baseline records them as new-coverage failures (probes 290 -> 298, 0 regressions).
