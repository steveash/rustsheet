---
id: T-0175
title: "Probe Excel's newer error literals (#BLOCKED!, #CONNECT!, #FIELD!, #BUSY!, #GETTING_DATA, #UNKNOWN!) in formulas and cells"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the error values Excel has beyond the classic eight plus `#SPILL!`/`#CALC!`:
whether `#BLOCKED!`, `#CONNECT!`, `#FIELD!`, `#BUSY!`, `#GETTING_DATA`, `#UNKNOWN!`, `#PYTHON!` and
`#EXTERNAL!` are accepted as error literals in formula text and as `t="e"` cell values, and how
they behave in ERROR.TYPE, ISERR/ISERROR/ISNA, IFERROR and comparisons. This is the evidence the
error-kinds spec (T-0177) and its engine tasks build on.

## Context
M1 scope includes error literals and error propagation. Today the engine has no error kind for
these (`Error` enum in crates/rustsheet-engine/src/expressions/token.rs), so 13 probe cells fail
(probes/rich-errors-field, -image, -stockhistory: the functions are #NAME? where Excel gives
#FIELD!/#BLOCKED!/#CONNECT!, ERROR.TYPE 13/11/10, and VALUETOTEXT the error name), and T-0124
needs `#BLOCKED!`. docs/excel-semantics/arrays.md (T-0030) maps rich-value errorType codes to
names and ERROR.TYPE values for #SPILL!, #CONNECT!, #BLOCKED!, #FIELD!, #CALC!; it records that
#BUSY!, #UNKNOWN! and other codes were never produced. What is not known: whether Excel's formula
grammar accepts `=#BLOCKED!` at all, and the ERROR.TYPE of the kinds never produced.

Risk: like literals of 1E+308 (numbers.md), an unknown token can make Excel for the web reject the
whole workbook (createSession 501 unsupportedWorkbook). Probe first with one single-cell workbook per
literal (`=#BLOCKED!`), then put the accepted literals together in
`eval/suites/probes/cases/error-literals-new.case.json` (excel-probe skill). Per accepted literal L:
`=L`, `=ERROR.TYPE(L)`, `=ISERR(L)`, `=ISNA(L)`, `=IFERROR(L,"x")`, `=VALUETOTEXT(L)`, `=L=L`,
`=#N/A=L` (which error wins), `=L+#DIV/0!` and `=#DIV/0!+L` (left-to-right precedence), lower case
`=#blocked!`. Also one cell per kind written as a `t="e"` value (`<v>#BLOCKED!</v>`, no formula)
with `=ERROR.TYPE(A1)` next to it; if spec2xlsx cannot write such a cell, note it as a harness gap
and skip that part.

## Acceptance criteria
- [ ] The single-literal results (accepted or rejected on load, with the oracle error message) are
      recorded in docs/excel-semantics/syntax.md; the probe case for the accepted literals and its
      excel-graph sidecar are committed.
- [ ] docs/excel-semantics/arrays.md table gains the ERROR.TYPE value of every literal Excel accepts
      (VERIFIED, citing the new cells); the precedence and comparison results are recorded.
- [ ] New-case engine mismatches are known failures in eval/baseline/probes.txt; no engine change.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine or facade support for the new kinds (spec T-0177, then its implementation tasks).
- IMAGE/STOCKHISTORY/FIELDVALUE values beyond their error (decision 0027: out of scope).
