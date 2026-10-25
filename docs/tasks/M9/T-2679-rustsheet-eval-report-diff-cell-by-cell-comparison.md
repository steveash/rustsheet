---
id: T-2679
title: "rustsheet-eval report-diff: cell-by-cell comparison of two eval reports for kernel ports"
milestone: M9
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2504]
spec: docs/specs/SPEC-0019-toms-708-bratio-incomplete-beta-port.md
---

## Goal
`rustsheet-eval report-diff <base report.json> <head report.json> [--suite S]` compares two eval runs cell by cell and
prints what a SPEC-0019 port task needs for decision 0115's check, per function (the specialised function the report
blames):
- fixed cells;
- lost cells (bit-exact before, not now), with their ulp distance;
- cells that changed but fail both times, with both ulp distances, flagging the ones that moved further from Excel;
- cells newly visibly off, or whose type or error changed;
- the fixed / lost ratio and the net change in visibly-off cells.

## Context
- SPEC-0019 port rule 2 and decision 0115. probes-numeric is ungated (`gate = false`), so `eval-check` never sees
  this family's main suite. Until now each port compared reports by hand (T-2502, T-2645).
- One definition. The verdicts and the visibly-off test come from `closeness.rs` (`Visible::of`, `is_off`,
  `within_fifteen_digits`) and `visible_parity.rs`. The fixed and lost sets come from `baseline::diff` over
  `Baseline::from_results` of the two runs. Nothing re-implements number comparison or blame (`closeness::blamed`).
- T-2505 and every later port task depend on it. T-2503 and T-2507, already in progress, may compare by hand until
  it lands.

## Acceptance criteria
- [ ] On two saved reports, the command prints the lists above per function, with ulp distances, and
      `--check-0115` exits non-zero when any function breaks decision 0115's thresholds.
- [ ] Its output, run on T-2502's merge (`main` before and after #1749), reproduces that Outcome's counts (154 fixed,
      0 newly failing).
- [ ] Unit tests cover each bucket and the per-function thresholds.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Running the eval: the user of the command runs `rustsheet-eval run` twice.
- Changing the gated `eval-check`.
