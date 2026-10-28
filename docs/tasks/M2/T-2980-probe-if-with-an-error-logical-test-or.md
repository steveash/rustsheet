---
id: T-2980
title: "Probe IF with an error logical_test or value_if_false, SUMIFS criteria that are zero or negative, and RIGHT with a numeric-text num_chars (M2 argument kinds T-2861 and T-2870 leave)"
milestone: M2
lane: research
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
Each value or criteria parameter of the M2 functions below has at least one Excel-backed cell for every argument kind
the report's argument coverage expects. That closes the last M2 argument-kind gaps, which neither T-2861 nor T-2870 lists.

## Context
Gap triage 2026-10-28 (`make eval` on d55479448, report.json `argument_coverage`; decision 0031 evidence gaps; decision
0094 asks for every argument mode to be covered). The kinds are those of
`crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative, fraction, numeric_text,
text, empty_text, boolean, error, blank, range. T-2861's Outcome names the first three gaps below as "kinds outside the
task's list still short". T-2870 (todo) owns only ABS.number empty text, MONTH.serial_number empty text and
COUNTIFS.criteria fraction or numeric text. These M2 parameters have no Excel cell of the listed kind in any suite:
- IF.logical_test: an error (covered 10 of 11).
- IF.value_if_false: an error (10 of 11).
- SUMIFS.criteria: zero and a negative number (9 of 11). Put both in the criteria and in the criteria range: `0` against
  a 0, a blank and a `"0"` text cell; `-2` against a -2 and a `"-2"` text cell. Also write each as a criterion string
  (`"0"`, `"-2"`, `"=-2"`, `"<0"`).
- RIGHT.num_chars: numeric text (10 of 11): `"2"`, `" 2 "`, `"2.7"`, `"1E1"`, plus a cell holding `"2"`.

Write one probe case, probes/m2-argument-kind-gaps-c (excel-probe skill, excel-graph, tag M2). Give every other argument
an ordinary valid value so that the probed argument decides the result. Write each kind in two forms where they differ:
a literal (`NA()`, `#DIV/0!`, `0`, `-2`, `"2"`) and a cell holding it (a formula cell `=NA()`, `=1/0`, a typed value).
For IF, an error in value_if_false matters only on the false branch, so test both a TRUE and a FALSE logical_test. Wrap
every error-valued cell in `ERROR.TYPE` too, so the error kind is scored. Existing rules to check against are in
docs/excel-semantics/criteria.md (numeric criteria matching text cells), logic.md and coercion.md.

## Acceptance criteria
- [ ] probes/m2-argument-kind-gaps-c is committed with oracle values (excel-graph) and covers every parameter/kind pair listed
      above. After `make eval`, report.json `argument_coverage` shows `covered == expected` for IF.logical_test,
      IF.value_if_false, SUMIFS.criteria (the zero and negative kinds; fraction, numeric text and the rest already covered)
      and RIGHT.num_chars.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's own
      module) or listed in a new functions-* task (functions-logic for IF, functions-criteria for SUMIFS, functions-text for
      RIGHT; campaign: semantics, with a `## Visible-parity stage` section) filed with `make new-task`, naming the case and
      cells. The new known failures are recorded with `make eval-baseline`, and each has that owner.
- [ ] Any new Excel rule is recorded in docs/excel-semantics/logic.md, criteria.md or text.md, citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The parameters T-2861 (done) and T-2870 (todo) list; M5 and later functions (T-2862 to T-2865, T-2871 to T-2878).
- Generator pools (T-2866, done).
