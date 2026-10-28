---
id: T-2870
title: "Probe ABS and MONTH of empty text and COUNTIFS criteria that are a fraction or numeric text (M2 argument kinds T-2861 leaves)"
milestone: M2
lane: research
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M2 functions below has at least one Excel-backed cell for each argument kind the
report's argument coverage expects, closing the M2 gaps T-2861 does not list.

## Context
Gap triage 2026-10-27 (`make eval` on 75a8a544b, report.json `argument_coverage`, decision 0031 evidence gaps;
decision 0094 asks for each argument mode to be covered). The kinds are those of
`crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative, fraction,
numeric_text, text, empty_text, boolean, error, blank, range. The generator's number pool has no empty text or
error literal (T-2866), so generated suites never reach most of these. `array`-mode parameters are not listed.
These M2 parameters have no Excel cell of the listed kinds in any suite and are not in T-2861's list:
- ABS.number: empty text.
- MONTH.serial_number: empty text (T-2861 lists DAY and YEAR only).
- COUNTIFS.criteria: a fraction (`0.5`, against a 0.5 in the criteria range) and numeric text (`"2"`, against a 2
  and a "2" in the criteria range). T-2861 lists SUMIF and SUMIFS only.

Write one probe case, probes/m2-argument-kind-gaps-b (excel-probe skill, excel-graph, tag M2).
Give every other argument an ordinary valid value, so the probed argument decides the result, write each kind
in two forms where they differ: a literal (`""`, `NA()`, `#DIV/0!`, `"2"`, `TRUE`) and a cell holding it (a formula
cell `=""`, `=NA()`), and wrap each error-valued cell in `ERROR.TYPE` too, so the error kind is scored.

## Acceptance criteria
- [ ] probes/m2-argument-kind-gaps-b is committed with oracle values (excel-graph) and covers every parameter/kind pair listed
      above; after `make eval`, report.json `argument_coverage` shows `covered == expected` for them.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's
      own module) or listed in a new functions-* task (campaign: semantics, with a `## Visible-parity stage`
      section) filed with `make new-task`, naming the case and cells; the new known failures are recorded with
      `make eval-baseline` and each has that owner.
- [ ] Any new Excel rule is recorded in docs/excel-semantics/coercion.md or criteria.md citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The parameters T-2861 lists (that task's probe); M5 and later functions (T-2862 to T-2865, T-2871 to T-2878).
- Generator pools (T-2866).
