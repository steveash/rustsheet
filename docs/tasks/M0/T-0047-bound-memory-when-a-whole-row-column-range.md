---
id: T-0047
title: "Bound memory when a whole-row/column range is lifted into an array (DATE(11:1899,12,1) OOM)"
milestone: M0
lane: engine
status: done
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
- [x] Excel's result for `=DATE(11:1899,12,1)` (and a whole-column variant such as `=ABS(A:A)`) is probed and recorded in `docs/excel-semantics/`.
- [x] Lifting a range into an array is bounded before cells are evaluated or recorded (no allocation proportional to the range area beyond a documented limit); a unit test covers the reproducer.
- [x] `fuzz/` `formula_eval` runs the reproducer under 2 GiB.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Every range lifted into an array now goes through one `Model::evaluate_range` (moved from model.rs
to cast.rs), which checks the shared array bound before reading any cell and reads whole-row/column
ranges only within the used area (the rest recorded read-empty in one rectangle). Scalar functions
(`get_number_or_array`), operators, IF/IFERROR, TEXT, IS*, NOT, spill functions, ARRAYTOTEXT,
TEXTSPLIT, SUMIF's criteria grid and a bare range spilled from a cell all use it. The bound is now
one whole column (2^20 cells, was 1,000,000), so whole-column lifts keep matching Excel.
Excel evidence: probes/lifted-range-size and probes/lifted-range-rows (excel-graph); Excel builds the
31M-cell `DATE(11:1899,12,1)` array (DEVIATION, docs/excel-semantics/arrays.md). The reproducer plus
the whole-column tests peak at ~194 MiB RSS (was > 2 GiB); cargo-fuzz is not installed locally, so the
`formula_eval` run itself is left to the nightly fuzz job.
Eval delta: probes fixed 2 (resource-limits C12, C14), new-coverage failures 4 (lifted-range-rows
C1-C3, E1, by design), regressions 0; lifted-range-size 6/6.
Follow-ups: T-0048 (clip whole-range walks), T-0049 (used-area tracking), T-0050 (move the bound to a core module).
