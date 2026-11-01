---
id: T-3550
title: "Gate A: exempt ABS, MIN and MAX as exact and SUMIF and SUMIFS as composite (the M2 functions without a table)"
milestone: M0
lane: harness
status: done
priority: P1
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
After this task, every M2 function that Gate A counts as missing has a table task (T-3555, T-3549) or an
`exempt.tsv` line. The five M2 functions that need no reference table get their line, backed by a decision record.

## Context
- `rustsheet-eval accuracy` on main (8ee20308) prints `missing tables 323`. Ten of them are M2 functions (M2 is the
  current milestone): ABS, AVERAGE, MAX, MIN, ROUND, ROUNDDOWN, ROUNDUP, SUM, SUMIF, SUMIFS. Count them with
  `eval/catalog/scope.json` (`scope: in`, `returns: number`) minus the functions named in `eval/accuracy/exempt.tsv`.
  T-3555 owns the three rounding functions and T-3549 owns SUM and AVERAGE. This task owns the other five.
- `eval/accuracy/exempt.tsv` classes (SPEC-0029 "Gate A holds"; T-3379):
  - `exact`: no floating-point kernel.
  - `composite`: a table of scalar literals cannot drive the function; the reason names the eval suites that do.
  The file is a protected path (decision 0199), so the change needs a decision record (`make new-decision`).
- Proposed classes. The reviewers check the reasoning and the cited evidence.
  - ABS, MIN, MAX: `exact`. The result is one of the argument doubles, or its negation, with no rounding.
    Coercion of the arguments is eval's (coercion.md "MIN / MAX coerce value arguments like SUM").
  - SUMIF, SUMIFS: `composite`. Their `range`, `sum_range` and `criteria_range` parameters are `mode: range` in the
    catalog, so a scalar-literal table cannot drive them. Their numeric part is a plain left-to-right sum that
    never snaps (numbers.md "Near-zero snapping…", T-3048 paragraph: SUMIF and SUMIFS keep 5.551115123125783e-17
    over 0.1, 0.2, -0.3). The suites to name are gen-m2, probes (e.g. probes/t3048-snap-conditional-sums) and
    ironcalc-calc. Before relying on any of them, confirm it has scored cells for both functions (`make eval`
    report, "Excel evidence per function").
  - If the evidence argues against one of these classes (for example, a scalar-literal form that Excel accepts),
    file a table task for that function instead of exempting it, and say so in the record.
- A line naming a function that has a table, or that is not an in-scope numeric catalog function, fails the check
  (exempt.tsv header). So this task must not touch the five functions of T-3555 and T-3549.

## Acceptance criteria
- [x] A decision record justifies each of the five lines, citing the catalog modes and the semantics entries above.
- [x] `eval/accuracy/exempt.tsv` gains ABS, MAX and MIN (`exact`) and SUMIF and SUMIFS (`composite`, naming the
      suites). `rustsheet-eval accuracy` reports `missing tables` 5 lower than before the change and no stale
      exemption. `scripts/accuracy_governance.py` accepts the change together with the record.
- [x] `make check` green; reviewers approve.

## Out of scope
- Tables for ROUND, ROUNDUP, ROUNDDOWN, SUM and AVERAGE (T-3555, T-3549).
- Exemptions for M5–M9 functions (T-3551 handles M5–M7; M9 is not filed yet).

## Outcome
Decision 0217 records the five lines. `eval/accuracy/exempt.tsv` now lists ABS, MAX and MIN as `exact` and SUMIF and
SUMIFS as `composite`. `rustsheet-eval accuracy` reports `missing tables` 318, down from 323, with no stale exemption.
The evidence table in the `make eval` report shows that gen-m2, probes and ironcalc-calc carry Excel cells for both
SUMIF and SUMIFS. Eval delta: none (no engine change). Follow-up filed: T-3551 (classify the M5–M7 functions with no table or exemption).
