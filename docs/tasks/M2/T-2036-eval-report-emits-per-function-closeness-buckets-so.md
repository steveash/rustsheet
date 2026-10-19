---
id: T-2036
title: "Eval report emits per-function closeness buckets so the parity page drops its regex and ulp copies"
milestone: M2
lane: harness
status: in-progress
priority: P3
size: S
tier: opus
depends_on: [T-2041]
---

## Goal
The eval report's per-function entries carry their closeness buckets (bit-exact, within 1 ulp,
within 2-4 ulp, within 1e-9, off), so the parity page generator reads them instead of
re-deriving them.

## Context
tools/src/rustsheet_tools/parity_report.py (T-2041) re-attributes the failing cells in
`cases[].cells` to functions with a copy of the `functions_in` regex in
crates/rustsheet-eval/src/report.rs, and splits `numeric_ulp` cells at 1 ulp with the ordered-bits
distance (`ulp_distance` in crates/rustsheet-eval/src/value.rs, mirrored in
tools/src/rustsheet_tools/fitlab/score.py). Both copies can drift from the harness. `FunctionStat`
in report.rs already counts verdicts per function; it can count the buckets in the same pass.

## Acceptance criteria
- [ ] report.json's `functions[]` entries gain bucket counts (pass with deviations, ulp1, ulp4,
  close, off; unsupported excluded) that sum to the scored cells, with a unit test in report.rs.
- [ ] parity_report.py reads those counts and drops its regex and failing-cell attribution; its
  output on the same report is unchanged (`make parity-report` shows no diff).
- [ ] The "mirrored in parity_report.py" comments in report.rs and value.rs are removed or updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing any verdict or the eval gate.
