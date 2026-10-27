---
id: T-2871
title: "Probe the M5 math argument kinds T-2862 leaves: empty text or a multi-cell range in DEGREES, EVEN, INT, POWER and SQRT"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M5 math functions below has at least one Excel-backed cell for each argument kind
the report's argument coverage expects.

## Context
Gap triage 2026-10-27 (`make eval` on 75a8a544b, report.json `argument_coverage`, decision 0031 evidence gaps;
decision 0094 asks for each argument mode to be covered). The kinds are those of
`crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative, fraction,
numeric_text, text, empty_text, boolean, error, blank, range. The generator's number pool has no empty text or
error literal (T-2866), so generated suites never reach most of these. `array`-mode parameters are not listed.
These M5 math parameters have no Excel cell of the listed kinds in any suite and are not in T-2862's list:
- DEGREES.angle: empty text, a multi-cell range.
- EVEN.number: empty text, a multi-cell range (T-2862 lists ODD's empty text only).
- INT.number: a multi-cell range.
- POWER.power: empty text.
- SQRT.number: empty text, a multi-cell range.
For a multi-cell range, write one formula in a row of the range and one outside its rows and columns (e.g.
`=INT(A1:A3)` in B2 and in D9), so both implicit-intersection outcomes are recorded.

Write one probe case, probes/m5-math-argument-kind-gaps-b (excel-probe skill, excel-graph, tag M5).
Give every other argument an ordinary valid value, so the probed argument decides the result, write each kind
in two forms where they differ: a literal (`""`, `NA()`, `#DIV/0!`, `"2"`, `TRUE`) and a cell holding it (a formula
cell `=""`, `=NA()`), and wrap each error-valued cell in `ERROR.TYPE` too, so the error kind is scored.

## Acceptance criteria
- [ ] probes/m5-math-argument-kind-gaps-b is committed with oracle values (excel-graph) and covers every parameter/kind pair listed
      above; after `make eval`, report.json `argument_coverage` shows `covered == expected` for them.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's
      own module) or listed in a new functions-* task (campaign: semantics, with a `## Visible-parity stage`
      section) filed with `make new-task`, naming the case and cells; the new known failures are recorded with
      `make eval-baseline` and each has that owner.
- [ ] Any new Excel rule is recorded in docs/excel-semantics/coercion.md (or numbers.md) citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The parameters T-2862 lists; M5 text (T-2863), M5 logic and information (T-2872).
- `array`-mode parameters (MMULT, MDETERM, MINVERSE, MULTINOMIAL).
