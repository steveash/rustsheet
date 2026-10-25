---
id: T-2660
title: "INDIRECT of a sheet-qualified sheet-scoped name (INDIRECT('Sheet!Name')) is #REF!, not the name's cell (calipers pv26_020)"
milestone: M6
lane: functions-lookup
status: done
priority: P1
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
`INDIRECT("<Sheet>!<Name>")` reaches a defined name scoped to that sheet, as Excel does.

## Context
Phase 1 assessment, 2026-10-25 (decision 0114). Real models use it to read one value per company
sheet. In `calipers/pv_goldens/pv26_020/golden.xlsx` (and its twin pv26_2_020), `Summary!C17`
`INDIRECT($B17&"!TargetName")` with B17 `Company0691` is `#REF!` in ours, `Company2277` in Excel;
D17 `!Announce` (41239), E17 `!Complete` (41303), F17 `!Currency` ("USD") likewise. The workbook
defines `TargetName` once per sheet (`localSheetId`), for example `Company0382!$D$24`. The `#REF!`
then reaches `MAX`, `MIN` and `AVERAGE(M7:M21)` on the same sheet: 50 visibly-off cells in the two
workbooks.

The calipers golden is desktop (Mac) evidence; Phase 1's oracle of record is excel-graph
(decision 0039), so probe it first with the `excel-probe` skill: a sheet-scoped name read through
INDIRECT with the sheet qualifier, without it from the name's own sheet and from another sheet,
quoted sheet names, a workbook-scoped name of the same name, and a name that is a range.

## Acceptance criteria
- [x] A probes case (milestone tag M6) with excel-graph values covers the forms above; the rule
      goes into docs/excel-semantics/references.md.
- [x] Ours matches the probe; the pv26_020 and pv26_2_020 cells are not visibly off.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns is within half a unit of the 15th significant digit of Excel's value, or
has the right type, error and text (decision 0083). Bit-level residue goes to a `defer: phase-1b`
follow-up (decision 0114).

## Out of scope
- INDIRECT with R1C1 text (unchanged).

## Outcome
- The engine already resolved `INDIRECT("<Sheet>!<Name>")` for sheet-scoped names. The pv26_020 and
  pv26_2_020 `Summary!*17` cells are a **corpus artifact**: the anonymisation renamed the text in
  `Summary!B17` to `Company0691` but left the sheet it names as `ConAgra` (whose `TargetName` D24
  is the cached `Company2277`). The shipped workbook has no `Company0691` sheet, so `#REF!` is the
  right answer for it; the 50 cells stay failing in the ungated calipers suite and are no longer
  actionable (lookup.md "INDIRECT of `Sheet!Name` text", calipers note).
- New probe probes/indirect-sheet-scoped-name (43 cells, excel-graph) found two real gaps, fixed
  in `indirect_defined_name`: an unquoted prefix that would need quotes (`"Co A!tn"`, `"P&L!tn"`,
  `"O'B!tn"`) is `#REF!`, and a quoted prefix undoubles `''` (`"'O''B'!tn"`). The prefix
  unquoting is shared with the parser (`unquote_sheet_prefix`).
- Eval delta: probes +42 passing cells of the new case (42/43); the remaining cell,
  `INDIRECT("2024!B1")`, is follow-up T-2678.
- The probe is tagged `M8`, not M6: defined-name probes take M8 (excel-probe skill, decision 0057),
  as its sibling probes/indirect-defined-name-text does.
