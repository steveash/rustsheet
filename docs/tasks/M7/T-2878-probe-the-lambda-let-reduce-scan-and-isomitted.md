---
id: T-2878
title: "Probe the LAMBDA, LET, REDUCE, SCAN and ISOMITTED argument kinds no Excel cell covers yet"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of LAMBDA, LET, REDUCE, SCAN and ISOMITTED listed below has at least one Excel-backed cell
for each argument kind the report's argument coverage expects.

## Context
Gap triage 2026-10-27 (`make eval` on 75a8a544b, report.json `argument_coverage`, decision 0031 evidence gaps;
decision 0094 asks for each argument mode to be covered). The kinds are those of
`crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative, fraction,
numeric_text, text, empty_text, boolean, error, blank, range. The generator's number pool has no empty text or
error literal (T-2866), so generated suites never reach most of these. `array`-mode parameters are not listed.
These M7 parameters have no Excel cell of the listed kinds in any suite:
- LAMBDA.calculation: every kind (a number, zero, a negative, a fraction, numeric text, text, empty text, a
  boolean, an error, a blank cell reference and a multi-cell range as the body of a LAMBDA called at once:
  `LAMBDA(x,"2")(1)`, `LAMBDA(x,A1:A3)(1)`).
- LET.calculation: an error.
- REDUCE.initial_value, SCAN.initial_value: an error, a multi-cell range (with `LAMBDA(a,b,a+b)` over {1,2,3}).
- ISOMITTED.argument: a multi-cell range (`LAMBDA(x,ISOMITTED(x))(A1:A3)`).
Read each array result through ROWS/COLUMNS and a TEXTJOIN of its cells, so the shape is scored and not only the top-left value.

Write one probe case, probes/m7-lambda-argument-kind-gaps (excel-probe skill, excel-graph, tag M7).
Give every other argument an ordinary valid value, so the probed argument decides the result, write each kind
in two forms where they differ: a literal (`""`, `NA()`, `#DIV/0!`, `"2"`, `TRUE`) and a cell holding it (a formula
cell `=""`, `=NA()`), and wrap each error-valued cell in `ERROR.TYPE` too, so the error kind is scored.

## Acceptance criteria
- [ ] probes/m7-lambda-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed
      above; after `make eval`, report.json `argument_coverage` shows `covered == expected` for them.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's
      own module) or listed in a new functions-* task (campaign: semantics, with a `## Visible-parity stage`
      section) filed with `make new-task`, naming the case and cells; the new known failures are recorded with
      `make eval-baseline` and each has that owner.
- [ ] Any new Excel rule is recorded in docs/excel-semantics/arrays.md (LAMBDA section) citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- An uncalled LAMBDA as a value (T-2622); a LAMBDA argument to lifted math functions (T-1919).
