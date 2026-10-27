---
id: T-2861
title: "Probe the M2 argument kinds no Excel cell covers yet: empty text, an error, a fraction or a boolean in 12 functions"
milestone: M2
lane: research
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M2 functions below has at least one Excel-backed cell for each argument kind the
report's argument coverage expects, so the current milestone's functions are verified on the inputs no suite
has tried yet.

## Context
Gap triage 2026-10-27 (`make eval` on 7bd9090e, report.json `argument_coverage`, decision 0031 evidence gaps;
decision 0094 asks for each argument mode to be covered). The argument kinds are those of
`crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative, fraction,
numeric_text, text, empty_text, boolean, error, blank, range. These M2 parameters have no Excel cell of the
listed kinds in any suite:
- DAY.serial_number, YEAR.serial_number: empty text.
- IF.logical_test: empty text.
- INDEX.row_num, INDEX.column_num: empty text.
- MATCH.match_type: empty text, error.
- VLOOKUP.range_lookup: empty text, error.
- HLOOKUP.row_index_num, HLOOKUP.range_lookup: empty text, error.
- LEFT.num_chars, RIGHT.num_chars: empty text, error.
- MID.text: error. MID.start_num: numeric text, empty text. MID.num_chars: empty text, error.
- SUMIF.criteria: a fraction, a boolean. SUMIFS.criteria: a fraction, numeric text, a boolean.

The generator's number pool (tools/src/rustsheet_tools/gen/generator.py `NUMBER_POOL`) has no empty text or
error literal, and the text pool's `"#N/A"` is text, which is why generated suites never reach these kinds.

Write one probe case, probes/m2-argument-kind-gaps (excel-probe skill, excel-graph, tag M2), with each kind in
two forms where they differ: a literal (`""`, `NA()`, `#DIV/0!`, `0.5`, `TRUE`, `"2"`) and a cell holding it
(a formula cell `=""`, `=NA()`, a typed `TRUE`). Wrap every error-valued cell in `ERROR.TYPE` too, so the error
kind is scored. For SUMIF/SUMIFS put the fraction and boolean both in the criteria and in the summed range
(0.5 against 0.5, TRUE against TRUE and against 1).

## Acceptance criteria
- [ ] probes/m2-argument-kind-gaps is committed with oracle values (excel-graph) and covers every
      parameter/kind pair listed above; after `make eval`, report.json `argument_coverage` shows
      `covered == expected` for the listed M2 parameters.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (if it is a one-line coercion fix in the
      function's own module) or listed in a new functions-* task (campaign: semantics, `## Visible-parity stage`)
      filed with `make new-task`, naming the case and cells; the new cells' known failures are recorded with
      `make eval-baseline` and each one has that owner.
- [ ] Any new Excel rule is recorded in docs/excel-semantics/ (coercion.md, lookup.md or criteria.md) citing
      the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Argument kinds of M5 and later functions (T-2862, T-2863, T-2864, T-2865).
- Generator pool changes (T-2866).
