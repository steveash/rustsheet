---
id: T-3535
title: "Gate A slice 3c: reference tables for the financial solvers (IRR, XIRR, NPER, RATE)"
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
Committed, oracle-filled Gate A tables for IRR, XIRR, NPER and RATE, each with a cited mpmath definition (RATE's
exists, T-3357), and `make check` running them.

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
- IRR and XIRR take ranges of flows: a table point is one call with array-literal flows (and dates for XIRR), each
  literal of at most 15 significant digits; several roots need the evidenced root choice or the point is redrawn
  (`NoValueError`).
- The decision 0188 assessment: IRR `#NUM!` near 0 and near −10% (T-3347), RATE `#NUM!` with fv ≠ 0 (T-3348), XIRR
  two-flow long spans 1e-8 to 1e-7 off (T-3353), NPER at a rate below 1e-14 up to 4% off where R3 says follow the math
  (T-3367: its points are R3-marked, with a decision record for `math-over-excel.tsv`).

## Acceptance criteria
- [ ] Definitions for IRR, XIRR and NPER, unit-tested against published values; tables for all four generated, filled
      and committed; each failing point is named with its owner task in the Outcome.
- [ ] NPER's below-1e-14 points are R3-marked and cited from financial.md's MATH-OVER-EXCEL entry.
- [ ] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The fixes for failing points (their owner tasks).
