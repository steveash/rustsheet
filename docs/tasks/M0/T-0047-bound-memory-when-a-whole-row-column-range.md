---
id: T-0047
title: "Bound memory when a whole-row/column range is lifted into an array (DATE(11:1899,12,1) OOM)"
milestone: M0
lane: engine
status: in-progress
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
A formula that lifts a whole-row or whole-column range into an array argument cannot exhaust
memory (CLAUDE.md directive 5), and returns what Excel returns.

## Context
P0 per the fuzz-crash rule (docs/process/agent-workflow.md): the nightly `formula_eval` job
is expected to stay red on this input until this task merges.
Found by the `formula_eval` fuzz target (T-0021): `=DATE(11:1899,12,1)` in a small workbook
exceeds libFuzzer's 2 GiB RSS limit. Rows 11:1899 are 1,889 × 16,384 ≈ 31M cells; `fn_date`
(`functions/date_and_time.rs`, via `get_number_or_array` in `cast.rs`) evaluates every cell, and
`Model::record_seen` (`evaluation.rs`) grows its seen-table for each one. The same shape likely
affects every scalar function that lifts a range argument element-wise. Reproduce:
`cd fuzz && cargo fuzz run formula_eval <file containing DATE(11:1899,12,1)>`.
What Excel returns for such a formula (a huge spill, so presumably `#SPILL!` in a cell without
room, or a resource error) needs an `excel-probe` before choosing the behaviour; a resource
limit that Excel lacks needs a `DEVIATION` entry (decision 0014).

## Acceptance criteria
- [ ] Excel's result for `=DATE(11:1899,12,1)` (and a whole-column variant such as `=ABS(A:A)`) is probed and recorded in `docs/excel-semantics/`.
- [ ] Lifting a range into an array is bounded before cells are evaluated or recorded (no allocation proportional to the range area beyond a documented limit); a unit test covers the reproducer.
- [ ] `fuzz/` `formula_eval` runs the reproducer under 2 GiB.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
