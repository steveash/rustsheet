---
id: T-0400
title: "Benchmark suite skeleton with synthetic scale models"
milestone: M3
lane: perf
status: blocked
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Criterion benchmarks for: full recalc of 100k/1M-formula chain and fan-out models, VLOOKUP-heavy and SUMIF-heavy sheets, single-cell edit + recalc, insert row in a 1M-cell sheet, memory per cell.

## Context
Baseline numbers from docs/research/ironcalc-assessment.md §2.3 (≈0.49 s per evaluate on 1M formulas).

## Acceptance criteria
- [ ] `make bench` runs them; results recorded in docs/research/benchmarks.md with machine info.
- [ ] Benchmarks build in CI (not run).
- [ ] `make check` green; reviewers approve.

## Out of scope
Optimizations.

## Blocked because
Deferred to Phase 2 (decision 0026): the dependency-graph, storage and performance work waits until
Phase 1 parity exits. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`.
