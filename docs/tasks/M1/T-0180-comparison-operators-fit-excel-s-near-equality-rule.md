---
id: T-0180
title: "Comparison operators: fit Excel's near-equality rule (2^-10 base distinguishes k>=3 ulps; 24 probes-numeric failures)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`=`, `<>`, `<`, `<=`, `>`, `>=` between two numbers decide "equal" exactly as Excel does for values a
few ulps apart, matching all 648 cells of the comparison-tolerance probe.

## Context
Rule (partly fitted): docs/excel-semantics/numbers.md, "Comparison tolerance" ("the threshold is left
to the implementing task"); measurements in docs/research/numeric-reaudit-2026-10-05.md (row
"Comparison tolerance"). Evidence: `probes-numeric/99bc3137b5869417` (excel-graph, 648 cells,
X = b·(1+k·2^-52) against b for 12 bases b in {2^-10, 0.25, 0.5, 1, 1.5, 2, 3, 4, 8, 100.5, 1024, 2^20}
and k = 1..8; Inputs!A-column holds the X values).

Failing today (BoolMismatch, 24 cells): Calc!A451, A453, A454, A456 (Inputs!A76, k = 3), A457, A459,
A460, A462 (k = 4), A463, A465, A466, A468 (k = 5), A469, A471, A472, A474 (k = 6), A475, A477, A478,
A480 (k = 7), A481, A483, A484, A486 (k = 8): all base 2^-10, operators `=`, `>`, `<=`, `<>`. Excel
says X ≠ 2^-10 for k ≥ 3 (e.g. A451 `Inputs!A76=(2^-10)` expected FALSE, actual TRUE); the engine
says equal.

Engine: `compare_values` in `crates/rustsheet-engine/src/functions/util.rs` rounds both sides to 15
significant digits (`to_excel_precision`) and then uses an absolute `f64::EPSILON`. 15-digit rounding
makes 2^-10·(1+3·2^-52) equal to 2^-10, which Excel does not; it happens to agree with Excel on the
other 11 bases. Fit a rule that reproduces all 648 cells (candidates: a relative-ulp threshold that
depends on the binary exponent/mantissa position, or Excel's own 15-digit cut applied differently);
explain the fit in numbers.md. `values_are_equal` and `CalcResult::cmp` (calc_result.rs) use other
tolerances; say in the Outcome whether they should follow (separate task if so, with evidence).
If the 648 cells do not pin the rule down, write a probe sweep (more bases and k, e.g. 2^-30, 2^-1,
10, 1e15) with the excel-probe skill before implementing.

probes-numeric is not gated (no baseline file): progress is read from `make eval`.

## Acceptance criteria
- [ ] In `make eval`, probes-numeric/99bc3137b5869417 has 0 failing cells (24 today).
- [ ] No new F lines in any gated baseline (gen-m1, gen-m2 comparisons and lookups are sensitive).
- [ ] numbers.md "Comparison tolerance" states the fitted rule with evidence and an
      `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Near-zero snapping of `+`/`-` (T-0110).
- Text collation (T-0220).
