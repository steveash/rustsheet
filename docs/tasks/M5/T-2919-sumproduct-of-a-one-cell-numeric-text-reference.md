---
id: T-2919
title: "SUMPRODUCT of a one-cell numeric-text reference beside a one-cell number is #VALUE! (poi FormulaEvalTestData_Copy.xlsx EverythingTests!J1328)"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
SUMPRODUCT gives Excel's result when an argument is a one-cell reference to numeric text ("4") beside other one-cell
arguments, starting with poi/FormulaEvalTestData_Copy.xlsx EverythingTests!J1328, the one unowned visibly-off
SUMPRODUCT cell in the functions-math lane.

## Context
Gap triage 2026-10-27 (`make eval` on origin/main f29502d9). poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, AppVersion
15.0300, i.e. Excel 2013, not the 16.x line decision 0030 calls desktop Microsoft 365):

| cell | formula | Excel | ours |
|---|---|---|---|
| EverythingTests!J1328 | `SUMPRODUCT(G1330:G1330,I1330:I1330)` | `#VALUE!` | 0 |

G1330 is the text "4" (a string cell, not a number), I1330 the number 2. The workbook's own "EXPECTED VALUE" row
(J1329) also says `#VALUE!`. The neighbours pass: K1328 `SUMPRODUCT(I1330:J1330,K1330:L1330)` and L1328
`SUMPRODUCT(K1330,J1330)` are `#REF!` (K1330 is `=AA10`, a `#REF!`), I1328 `SUMPRODUCT(2,C10:C11)` is `#VALUE!`.
It is an F line in eval/baseline/poi.txt.

This looks like a different rule from the one already recorded: docs/excel-semantics/coercion.md "SUMPRODUCT: a blank
one-cell argument beside other arguments is `#VALUE!` (T-2150)" says, from probes/t2200-blank-and-padded-m5-args
(excel-graph, 2026-10-21) A10, A11, A14, that a one-cell reference to the non-numeric text "x" is a 0 factor beside a
number reference (either order) and alone. No oracle cell in eval/ covers a one-cell reference to *numeric* text beside
a one-cell number (checked across every suite's .oracle.json on this run: the other SUMPRODUCT cells with a
numeric-text one-cell argument also mix in a shape mismatch, which is `#VALUE!` anyway). So either current Excel
treats numeric text "4" differently from "x" (the J1328 reading), or Excel 2013 differed and current Excel gives 0.
The quirk is noted, UNVERIFIED, in coercion.md under the T-2150 entry.

Steps:
1. Probe with the `excel-probe` skill (excel-graph): one-cell numeric text ("4", " 4", "1.5", "1E3", "$4"),
   non-numeric text ("x") and a numeric-text formula result (`="4"`), each as a cell reference and as a one-cell range
   (`G1:G1`), beside a one-cell number (both orders), beside a scalar constant, beside a 1x1 array `{2}`, alone, and in
   a 2x1 range beside a 2x1 number range (an element, not a one-cell argument). Include the J1328 shape verbatim.
2. If excel-graph says `#VALUE!` for J1328's shape, implement the rule in `fn_sumproduct` / `sumproduct_array`
   (crates/rustsheet-engine/src/functions/math_and_trigonometry/sumproduct.rs), next to the blank one-cell rule.
3. If excel-graph says 0, current Excel and the 2013 cache disagree: J1328 is old-version evidence (decision 0030 makes
   only 16.x desktop the reference). Record that in coercion.md and handle the cell as decision 0030 / 0019 say for
   evidence that contradicts the web oracle (the probe cells become the evidence; ask the Excel-fidelity reviewer how
   to retire the F line, never hand-edit the baseline).

## Acceptance criteria
- [ ] A committed probe case (probes suite, oracle-filled from excel-graph) covers the shapes in step 1, and every
      cell of it passes.
- [ ] The poi/FormulaEvalTestData_Copy.xlsx EverythingTests!J1328 line disappears from eval/baseline/poi.txt
      (`make eval-baseline`) with no new F line in any baseline, or, under step 3, the task records the decision that
      retires it and the evidence.
- [ ] coercion.md's SUMPRODUCT entry states the numeric-text rule as VERIFIED (excel-graph), citing the probe cells and
      J1328, and drops the UNVERIFIED note.
- [ ] Unit tests in sumproduct.rs for the numeric-text one-cell argument beside a number, in both orders.
- [ ] `make check` green; reviewers (including Excel fidelity) approve.

## Out of scope
- Element text inside multi-cell ranges (already 0 and passing).
- Other aggregates' handling of one-cell text references (file follow-ups if the probe shows a shared rule).

## Visible-parity stage
poi/FormulaEvalTestData_Copy.xlsx EverythingTests!J1328 and every probe cell have Excel's type and error (stage
`semantics`: a wrong type caused by an argument rule).
