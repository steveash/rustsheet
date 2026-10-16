---
id: T-1641
title: "A sheet-qualified name falls back to the workbook-scoped name when the sheet has no local one (Feb!loc)"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`Feb!loc`, where Feb has no sheet-scoped `loc`, is the workbook-scoped `loc`, as in Excel.

## Context
gen-m8a (T-1606) names-scope c238d93c128fd5ee and names-errors 6322c0f2638aa11b: `=IFERROR(Feb!loc,"e")` and `=IFERROR(Other!loc,"e")` are 10 in Excel (workbook `loc` = 10; Calc!loc and Jan!loc exist), an error in the engine; `ISERROR(Feb!loc)` is FALSE. A qualified name with no local definition and no workbook definition is still `#NAME?` (`Other!only1` is `#NAME?`, `ISERROR` TRUE, `IFERROR(Other!only1,x)` = x; the engine already matches these). docs/excel-semantics/references.md "A sheet-qualified name falls back to the workbook name".

## Acceptance criteria
- [ ] The `Feb!loc` / `Other!loc` probes of gen-m8a names-scope and names-errors pass, with no regression elsewhere.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`prefixed_name_operand` (parser/operands.rs) now falls back to the workbook-scoped name when a
sheet-qualified name has no local definition; with neither, it stays `#NAME?`. Unit test
`sheet_qualified_name_falls_back_to_workbook_name`. gen-m8a cases c238d93c128fd5ee and
6322c0f2638aa11b no longer appear in the failure report; no baseline change (gen-m8a is not
baselined), no regressions.
