---
id: T-3553
title: "Gate A slice 3b: reference tables for the discrete distributions (BINOM.DIST, NEGBINOM.DIST, POISSON.DIST)"
milestone: M0
lane: harness
status: in-progress
priority: P0
size: M
tier: opus
depends_on: [T-3357, T-3379]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Committed, oracle-filled Gate A tables for BINOM.DIST, NEGBINOM.DIST and POISSON.DIST (both the cumulative and the
mass forms), each with a cited mpmath definition, and `make check` running them.

## Context
- Split out of T-3358 (2026-10-31): that task named 18 functions in one batch, more than an M task, and only T.DIST,
  T.INV and RATE had definitions (T-3357). T-3358 keeps the continuous distributions and quantiles.
- SPEC-0029 ("The generator and the sampler", "Point fields", "The check"); decision 0188 R2 and R3. Generate with
  `cd tools && uv run rustsheet-accuracy generate <FUNCTION>` after adding a `Definition` under
  `tools/src/rustsheet_tools/accuracy/defs/`, fill with `uv run rustsheet-oracle fill -b excel-graph --suite accuracy`,
  and check with `rustsheet-eval accuracy`.
- A failing point is carried by a `known-failures.tsv` line naming its owner task, only for points added in the same
  diff (T-3379); never by weakening ε or the 15-digit rule.
- After the definitions land, rerun `uv run rustsheet-accuracy math --suite probes-numeric` (T-3380) and commit the
  sidecars, so R1(c) sees these functions' probes-numeric cells.
- The decision 0188 assessment found BINOM.DIST at n ≥ 6e5 and NEGBINOM.DIST at a large number of failures 1e-9 to
  1e-8 off (T-3352 owns them); POISSON.DIST and BINOM.DIST generalize poorly to fresh points (T-3366). Their failing
  points are carried with those owners.

## Acceptance criteria
- [ ] Definitions for the three functions, unit-tested against published values; tables generated, filled and
      committed; each failing point is named with its owner task in the Outcome.
- [ ] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The fixes for failing points (their owner tasks).
