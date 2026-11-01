---
id: T-3535
title: "Gate A slice 3c: reference tables for the financial solvers NPER and RATE (IRR and XIRR split to T-3586)"
milestone: M0
lane: harness
status: done
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
- [x] Definitions for IRR, XIRR and NPER, unit-tested against published values; tables for all four generated, filled
      and committed; each failing point is named with its owner task in the Outcome. (Split: IRR and XIRR need
      array-literal arguments, now T-3586; this task delivers NPER and RATE.)
- [x] NPER's below-1e-14 points are R3-marked and cited from financial.md's MATH-OVER-EXCEL entry.
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [x] `make check` green; reviewers approve.

## Out of scope
- The fixes for failing points (their owner tasks).

## Outcome
Done (2026-11-01).
- **Split:** this task delivers NPER and RATE. IRR and XIRR are filed as T-3586: a Gate A argument is a number, an
  integer or a logical, so their flows need array-literal arguments in the generator first.
- **NPER definition** (`tools/src/rustsheet_tools/accuracy/defs/annuity.py`):
  - The closed form, with -(pv + fv)/pmt at rate 0.
  - Excel's errors (financial.md, "Annuity and DB argument domains at their limits"): a non-positive ratio and
    rate ≤ -1 are `#NUM!`, and a rate where 1 + rate is 1 is `#DIV/0!`.
  - A whole-annuity draw like RATE's sets pmt so the drawn period count is the root. Its tail stratum covers R3's
    tiny positive rates.
  - Tests against Microsoft's examples are in `tools/tests/test_accuracy_nper.py`.
- **Tables:** `s1` batches for NPER (300 points) and RATE (300, its first table; the definition is T-3357's), filled
  from excel-graph.
- **Check:** every point is inside the envelope except 19 known failures, each with an owner:
  - **T-3367**, 12 points: the R3-marked NPER points (decision 0220), where the engine still reproduces Excel. They are
    A121, A130, A133, A135, A142, A143, A151, A152, A153, A156, A158 and A162.
  - **T-3585** (new), 7 points: RATE is `#NUM!` in Excel where the math has a root. A19, A128, A129, A137 and A158
    are type-1 annuities at a high rate, where the engine returns a bogus root near -1. A278 and A291 have a huge pv
    and a root near 0, where the engine returns the math.
- **R3 (decision 0220):** the 12 NPER points sit inside decision 0188's approved region. Excel is up to 35% off there,
  not the 4% the assessment found, and financial.md's entry now says so and cites them. The other NPER points with
  Excel more than G off are ill-conditioned annuities outside the region; the engine matches Excel to the bit there.
- **probes-numeric math sidecars:** regenerated with the NPER definition.
