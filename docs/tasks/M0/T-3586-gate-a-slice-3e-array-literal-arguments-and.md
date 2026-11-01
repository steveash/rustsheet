---
id: T-3586
title: "Gate A slice 3f: array-literal arguments and reference tables for IRR and XIRR"
milestone: M0
lane: harness
status: done
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
- [x] Array-literal arguments are written, read back and evaluated, with unit tests.
- [x] Definitions for IRR and XIRR are unit-tested against published values. Their tables are generated, filled and
      committed, and the Outcome names each failing point with its owner task.
- [x] The probes-numeric math sidecars are regenerated and committed.
- [x] `make check` green; reviewers approve.

## Out of scope
- The fixes for failing points (their owner tasks).

## Outcome
Done (2026-11-01).
- **Array-literal arguments** (`tools/src/rustsheet_tools/accuracy/`):
  - `literal.Arg` gains a tuple of numbers, written as `{a,b,…}` with 15-digit elements.
  - A Param kind `"array"`; `model.MathArg` holds the elements; `check_domain` leaves arrays to the definition.
  - The sampler's `finish` writes each element: integers stay integers.
  - `tables.parse_formula` reads array literals back for `verify`.
  - `eval_math.fit_arg` reads a range or array argument as its numbers in row-major order, so probes-numeric cells
    over ranges get math values.
- **Definitions** (`defs/cashflow.py`):
  - IRR and XIRR are the unique root of the exponential sum sum c_i (1 + r)^(-t_i) for time-ordered flows with one
    sign change (Descartes' rule). Flows of one sign are a domain error; several sign changes redraw.
  - Each draw builds the series around a drawn rate. It covers rates near 0 and near -10% (T-3347), long series, and
    XIRR's two flows over long spans (T-3353).
  - Tests against Microsoft's examples are in `tools/tests/test_accuracy_cashflow.py`. Microsoft's published XIRR,
    0.373362535, is 1.5E-9 above the true root, Excel's early stop.
- **Tables:** `s1` batches for IRR and XIRR, 300 points each, filled from excel-graph.
- **Check:** every XIRR point is inside the envelope, including the long two-flow spans. 23 IRR points are known
  failures, all on conventional series:
  - **T-3347**, 6 points: roots near 0 where the engine is `#NUM!` and Excel returns the root (A271, A280, A286, A288,
    A290, A298).
  - **T-3548**, 17 points:
    - the engine `#NUM!` where Excel has the root: A190, A193, A231, A259, A260 and A264;
    - Excel `#NUM!` where the engine returns the root: A216, A221, A233, A238, A240, A242, A258 and A270;
    - Excel `#DIV/0!` where the engine is `#NUM!`: A237, A248 and A268.
  - financial.md's "IRR is #NUM! or #DIV/0! on some conventional series it could solve" cites the Excel errors.
- **probes-numeric math sidecars:** regenerated, so IRR and XIRR cells over ranges have math values.
