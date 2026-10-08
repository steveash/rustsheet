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
Probe `probes/rate-empty-arguments` (10 cells, excel-graph): Excel reads an empty guess as 0, not 0.1 (A1 = A3 = A8 bit for bit, A2 = A4), an empty `type` as 0 and `TRUE` as guess 1 (A9 = A10). Findings in docs/excel-semantics/financial.md. No code change: the engine returns `#NUM!` for A1, A3 and A8 alike (guess 0 does not converge), so it is consistent with reading the empty guess as 0, but the convergence and last-digit differences (A1-A6, A8-A10) are left to T-0916. Only A7 (`""` is `#VALUE!`) passes; the others are baselined as new-coverage failures, 0 regressions.
