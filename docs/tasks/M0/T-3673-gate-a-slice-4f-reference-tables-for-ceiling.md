---
id: T-3673
title: "Gate A slice 4f: reference tables for CEILING, CEILING.MATH, CEILING.PRECISE, FLOOR, FLOOR.MATH and FLOOR.PRECISE"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-3551, T-3555]
---

## Goal
CEILING, CEILING.MATH, CEILING.PRECISE, FLOOR, FLOOR.MATH and FLOOR.PRECISE each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- Reuse the decimal-math rule of T-3555 (the written 15-digit decimal rounded exactly), so the rounding family shares
  one reading of "the math" of a rounding function.
- The sign rules of CEILING and FLOOR with a negative significance (Excel's legacy `#NUM!` for opposite signs) and the
  mode argument of the .MATH forms are domain rules to cite from the semantics entries (search numbers.md for
  CEILING/FLOOR before writing the domain).
- A huge number over a tiny significance may overflow or lose the quotient's digits; such points are R3 candidates.
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [x] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [x] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 6. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

## Blocked because
Resumed after the review-round limit (step 5c).
- PR #2792 (draft). security, architecture and quality APPROVE on 2692ee19 (round 1). excel-fidelity asked for
  changes in all three rounds; each round's findings are fixed on the branch, the last ones in the commit after
  e291348c:
  - round 1: the overflow bracket claim in numbers.md was wrong (now 0.556x to 49x DBL_MAX); FLOOR(0,0) had no
    evidence (now NoValueError).
  - round 2: plain FLOOR's overflow `#NUM!` had no FLOOR cell (added FLOOR/e-T-3673, 5 overflow cells, all `#NUM!`);
    the result-overflow DomainError was unevidenced (now NoValueError); decision wording for A170/A173.
  - round 3: the binary-quotient cut needed its own numbers.md entry (added "CEILING and FLOOR cut the binary
    quotient, so CEILING can fall below its argument"); the plain-ceiling figure for CEILING.MATH/s1 A81 is
    701.0101010101009; ratios in the unpinned band 0.5563x to 48x DBL_MAX now have no math value.
- Next: per work-loop step 5c, run `make check` and the full panel on the current head; no reviewer found a defect
  in the definitions or the tables, all findings were in the docs and in unevidenced corners.

## Outcome
- Definitions: `tools/src/rustsheet_tools/accuracy/defs/ceiling_floor.py`, one per function. The math reads the written
  decimals and takes Excel's 15-digit ratio cut (ties away from zero) before the step, as decision 0223 does for the
  ROUND family; decision 0244 records the reading and why a plain ceiling would put false R3 marks a whole step off
  (`CEILING(100,1.01010101010101)`: 99.99999999999999, not 101.010101010101). Unit tests in
  `tools/tests/test_accuracy_ceiling_floor.py` against the numbers.md tie, sign, zero and subnormal-ratio cells.
  `decimal_literal` and `random_sign` moved from `defs/rounding.py` to `sampler.py` so both families share them; the
  ROUND tables regenerate byte-identical.
- Tables: `eval/accuracy/tables/{CEILING,CEILING.PRECISE,FLOOR,FLOOR.PRECISE}/s1` (120 points each) and
  `{CEILING.MATH,FLOOR.MATH}/s1` (200 points each, with a whole-number `mode`), filled from excel-graph on 2026-11-01.
  `FLOOR/e-T-3673` (60 log-uniform points) evidences FLOOR's overflow `#NUM!` (Excel-fidelity review).
  `rustsheet-eval accuracy`: 4056 points in 22 tables, 0 failing, 0 table defects; missing tables 277 to 271. No
  known-failures lines: every new point is in the envelope.
- All 41 math domain errors (32 in s1, 9 in FLOOR/e-T-3673) are errors of the same code in Excel (CEILING/FLOOR `#NUM!` for a positive number over a
  negative significance, FLOOR `#DIV/0!` for a zero significance, and `#NUM!` at all 20 points whose ratio overflows, between 0.556 and 49 times the double range at the boundary).
  FLOOR(0,0) and a result beyond the double range have no math value: no Excel cell evidences them.
  The overflow rule was not recorded: new numbers.md entry "CEILING and FLOOR are #NUM! when number/significance
  overflows" (VERIFIED, the engine already matches).
- Excel is more than G = 1e-4 off the math (relative, no floor) at no point: no R3 candidate. Four points are one unit
  of the cut's 15th digit off the math (CEILING/s1 A58, CEILING.MATH/s1 A170, FLOOR.MATH/s1 A99, A173; at most 4.9e-15),
  where Excel cuts its binary quotient; the engine matches Excel to the bit there.
- `_near_step`'s repeating-significance shape has few choices, so three s1 pairs repeat (CEILING A53/A55,
  CEILING.MATH A103/A110, CEILING.PRECISE A49/A51); harmless, left as is since a batch is never rewritten.
- `rustsheet-accuracy math --suite probes-numeric` changed no committed sidecar (no probes-numeric cell is a one-call
  CEILING/FLOOR with literal arguments); the unrelated new sidecars it wrote were left out, as in T-3555.
