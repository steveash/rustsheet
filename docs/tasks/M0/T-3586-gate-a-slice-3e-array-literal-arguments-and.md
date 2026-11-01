---
id: T-3586
title: "Gate A slice 3f: array-literal arguments and reference tables for IRR and XIRR"
milestone: M0
lane: harness
status: todo
priority: P0
size: M
tier: opus
depends_on: [T-3535]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Committed, oracle-filled Gate A tables for IRR and XIRR, each with a cited mpmath definition. To get there, the generator
learns array-literal arguments (a range of flows, and dates for XIRR), and `make check` runs the new tables.

## Context
- Split out of T-3535 (2026-11-01). That task kept NPER and RATE. A Gate A argument today is a number, an integer or a
  logical (`literal.Arg`), so the flows of IRR and XIRR cannot be written.
- The work:
  - Extend `literal.Arg` with a tuple of numbers, written as an array literal `{a,b,…}` with each element at most 15
    significant digits.
  - Teach the sampler, the case writer, `sampler.math_arg` and `eval_math` (the probes-numeric sidecars) to handle it.
  - Add a whole-point draw for each function, like RATE's (`defs/annuity.py`): draw a rate and flows that make it the
    root.
  - Where there are several roots, use the evidenced root choice or redraw the point (`NoValueError`).
- The decision 0188 assessment:
  - IRR is `#NUM!` near 0 and near −10% (T-3347).
  - XIRR with two flows over long spans is 1e-8 to 1e-7 off (T-3353).
  - Failing points are carried with those owners.
- SPEC-0029 ("The generator and the sampler", "Point fields", "The check"); decision 0188 R2.
- After the definitions land, rerun `uv run rustsheet-accuracy math --suite probes-numeric` (T-3380) and commit the
  sidecars.

## Acceptance criteria
- [ ] Array-literal arguments are written, read back and evaluated, with unit tests.
- [ ] Definitions for IRR and XIRR are unit-tested against published values. Their tables are generated, filled and
      committed, and the Outcome names each failing point with its owner task.
- [ ] The probes-numeric math sidecars are regenerated and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The fixes for failing points (their owner tasks).
