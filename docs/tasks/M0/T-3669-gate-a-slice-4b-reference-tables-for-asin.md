---
id: T-3669
title: "Gate A slice 4b: reference tables for ASIN, ACOS, ATAN, ATAN2, ACOT, DEGREES and RADIANS"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3551]
---

## Goal
ASIN, ACOS, ATAN, ATAN2, ACOT, DEGREES and RADIANS each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- numbers.md, "ACOS is PI()/2 - ASIN(x)": Excel's ACOS is computed through ASIN, so near x = 1 it may be far from the math
  (an R3 candidate needing an owner ruling, or a definition that reads Excel's formula if the entry shows it is a
  semantic rule).
- numbers.md, "Out-of-domain ACOS/ASIN/ACOSH/ATANH are `#NUM!` at once": the domain is [-1, 1].
- ATAN2 takes (x, y) in Excel's order, and ATAN2(0,0) is `#DIV/0!`.
- numbers.md, "RADIANS, SQRTPI and MUNIT": RADIANS is `x * (PI/180)` to the bit; DEGREES is not yet in an entry.
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [x] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [x] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 7. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [x] The probes-numeric math sidecars are regenerated with the new definitions (no probes-numeric case holds a
      one-call cell of these functions with literal arguments, so none changed).
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

## Outcome
Done (2026-11-01).
- **Definitions** (`tools/src/rustsheet_tools/accuracy/defs/inverse_trig.py`): the true functions of the doubles.
  - ASIN and ACOS take [-1, 1]. Excel's ACOS goes through its ASIN in doubles, a kernel formulation, so the definition
    is the true arccosine.
  - ACOT is π/2 - atan(x), with range (0, π): gen-m5 has `ACOT(-0.1)` = 1.6704649792860584 (A278).
  - DEGREES and RADIANS are x·180/π and x·π/180.
  - ATAN2(x_num, y_num) is atan2(y, x). `ATAN2(0,0)` (`#DIV/0!`) and a quotient beyond the double range (`#NUM!`) are
    domain errors. A quotient that underflows is 0 in Excel, which moves the result by less than the smallest normal
    double, inside Gate A's absolute floor (as decision 0223 relies on), so the definition has no rule for it.
  - Draws: ASIN and ACOS near ±1 and 0; the reals at tiny and huge magnitudes; ATAN2 across quadrants, axes, and
    quotients near overflow and underflow.
  - Tests: `tools/tests/test_accuracy_inverse_trig.py`.
- **numbers.md** "ACOS is PI()/2 - ASIN(x)": corrected. Excel's `ACOS(0.5)` = 1.0471975511965976 is one ulp below
  the double nearest π/3; libm's 1.0471975511965979 is the correctly rounded value (1.07E-16 from π/3 against 1.15E-16).
- **Tables:** `s1` for the seven functions, 120 points each, filled from excel-graph.
  - All 840 points are inside the envelope: 0 failing, 0 table defects, no known failures added.
  - `missing tables` drops by 7 (277 to 270).
  - Every domain-error point is an Excel error, and every number point is an Excel number.
  - Excel is more than G = 1e-4 off the math at five points, all subnormal true values that Excel shows as 0, inside
    Gate A's absolute floor: `RADIANS(2.22507385850721E-308)` (A83, true 3.9E-310) and `ACOT(9.99999999999999E+307)`
    (ACOT A70, A73, A74, A78, true 1.0E-308). Graph reports a kept subnormal as 0 too and the tables have no `(x)=0`
    companion, so whether Excel's 0 is a true 0 is not told here. Elsewhere the worst point is 2.5E-16 relative.
  - ACOT's math is acot(x) on (0, π), not π/2 - atan(x): that form cancels to 0 beyond about 1e80 even at 80 digits,
    and the excel-fidelity review caught 15 such zeros in the first draft of the table. The regenerated case is
    byte-identical, so the oracle fill stands.
