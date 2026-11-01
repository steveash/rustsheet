---
id: T-3672
title: "Gate A slice 4e: reference tables for EXP, LN, LOG, LOG10, POWER, SQRT and SQRTPI"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-3551]
---

## Goal
EXP, LN, LOG, LOG10, POWER, SQRT and SQRTPI each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- numbers.md, "POWER and `^`: odd roots of negatives, zero and overflow": POWER's domain rules (a negative base with a
  non-integer exponent, 0^0, 0 to a negative power, overflow).
- numbers.md, "RADIANS, SQRTPI and MUNIT": SQRTPI is `sqrt(x * PI)` to the bit.
- LOG's base defaults to 10; base 1 is `#DIV/0!` even for `LOG(1,1)`, and base 0 or a negative base is `#NUM!`
  (numbers.md, the LOG entry around "`#DIV/0!` even for `LOG(1,1)`").
- EXP's overflow cut (about 709.78) is a domain edge worth an edge point.
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [x] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [x] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 7. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

## Outcome
- **Definitions** (`tools/src/rustsheet_tools/accuracy/defs/exp_log.py`): the true function of the doubles of the
  written literals. LOG is the two-argument form (one-argument LOG is LOG10's double).
  - EXP: `#NUM!` above ln(DOUBLE_MAX); the unpinned cut band (709.78, 709.79) is kept out of the draws.
  - LN, LOG10: a positive argument. LOG: positive number and base, base 1 `#DIV/0!` (even `LOG(1,1)`).
  - SQRT: a non-negative argument. SQRTPI: the same, and `#NUM!` where x·π overflows (band around 5.72E+307 kept out).
  - POWER, from the numbers.md entries:
    - 0^0 is `#NUM!`, 0 to a negative power `#DIV/0!`;
    - the negative-base odd-root rule (1/y at 15 digits odd and below 2^31 - 1);
    - a negative base with an integer exponent of 2^32 - 1 or more is `#NUM!`;
    - the positive power's overflow (`#NUM!`) and underflow under a negative exponent (`#DIV/0!`);
    - the T-2661 product overflow.
    Draws keep the positive power at least 1e-6 (relative, in its log) from both range edges.
  - Excel's kernels (x87 EXP/LN/LOG10, square-and-multiply, exp(y·ln x)) are kernel detail, not semantic rules.
  - Tests: `tools/tests/test_accuracy_exp_log.py`.
- **Tables:** `s1` for the seven functions, 120 points each, filled from excel-graph.
  - All 840 points are inside the envelope: 0 failing, 0 table defects, no known failures added.
  - `missing tables` drops by 7 (277 to 270).
  - Every domain-error point is the matching Excel error: POWER's 12 (`#NUM!` for 0^0, odd-root misses, the 2^32 - 1
    limit, overflow and the T-2661 product; `#DIV/0!` for 0^-1 and the underflowing positive power), LOG's 13 (`#DIV/0!`
    for base 1, `#NUM!` for base 0 or negative), EXP, LN, LOG10, SQRT and SQRTPI's `#NUM!`.
  - Excel is within 2.6E-14 relative of the math at every normal true value (POWER's exp(y·ln x) at huge or tiny
    bases; the other functions within 2.1E-16). It is more than G off at 15 EXP points from -714 to -1000, all
    subnormal or smaller true values that Excel shows as 0, inside the absolute floor. No R3 candidate.
  - New evidence in numbers.md: the odd-root cut for a negative exponent and for 1/9; EXP's sub-normal-range points.
- **probes-numeric math sidecars:** 25 files gain cells of these functions (15350 numeric cells plus 233 errors; three
  t2586 F.DIST sidecars only gain EXP cells). Every one is within G of Excel or inside the floor (EXP and POWER results
  below the smallest normal double, which Excel shows as 0).
  - Overlaps with open PRs: double-rounding-chained and double-rounding-ops (#2702, SUM and AVERAGE cells) and
    power-ulp-sources (#2764, TANH cells) are new files there too. Whichever PR merges later regenerates them with
    all definitions.
