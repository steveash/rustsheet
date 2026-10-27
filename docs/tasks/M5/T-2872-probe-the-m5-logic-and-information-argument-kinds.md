---
id: T-2872
title: "Probe the M5 logic and information argument kinds no Excel cell covers yet: IFS, SWITCH, ISERR, ISEVEN, ISODD, INFO and ENCODEURL"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M5 logic and information functions below has at least one Excel-backed cell for
each argument kind the report's argument coverage expects.

## Context
Gap triage 2026-10-27 (`make eval` on 75a8a544b, report.json `argument_coverage`, decision 0031 evidence gaps;
decision 0094 asks for each argument mode to be covered). The kinds are those of
`crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative, fraction,
numeric_text, text, empty_text, boolean, error, blank, range. The generator's number pool has no empty text or
error literal (T-2866), so generated suites never reach most of these. `array`-mode parameters are not listed.
These M5 parameters have no Excel cell of the listed kinds in any suite (no other probe task lists them):
- IFS.logical_test: empty text. IFS.value_if_true: an error (returned, and skipped when its test is FALSE).
- SWITCH.result, SWITCH.default: an error (returned, and not reached).
- ISERR.value: empty text. ISEVEN.number, ISODD.number: empty text.
- INFO.type_text: a number, zero, a negative, a fraction, numeric text, a boolean, an error, a blank and a
  multi-cell range. None of these is a valid info type, so the result should not depend on the machine; if one is
  not an error in Excel, record it and leave it out of the case.
- ENCODEURL.text: an error.
FILTERXML is left out: its Excel evidence is web-only so far (T-1081).

Write one probe case, probes/m5-logic-argument-kind-gaps (excel-probe skill, excel-graph, tag M5).
Give every other argument an ordinary valid value, so the probed argument decides the result, write each kind
in two forms where they differ: a literal (`""`, `NA()`, `#DIV/0!`, `"2"`, `TRUE`) and a cell holding it (a formula
cell `=""`, `=NA()`), and wrap each error-valued cell in `ERROR.TYPE` too, so the error kind is scored.

## Acceptance criteria
- [ ] probes/m5-logic-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed
      above; after `make eval`, report.json `argument_coverage` shows `covered == expected` for them.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's
      own module) or listed in a new functions-* task (campaign: semantics, with a `## Visible-parity stage`
      section) filed with `make new-task`, naming the case and cells; the new known failures are recorded with
      `make eval-baseline` and each has that owner.
- [ ] Any new Excel rule is recorded in docs/excel-semantics/coercion.md (or logic.md, information.md) citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- M5 math (T-2862, T-2871) and text (T-2863); the DBCS `*B` text variants (T-2863 leaves them out).
- FILTERXML (T-1081).
