---
id: T-1386
title: "Probe Excel's formula length limit for typed formulas and defined names (8,192 characters)"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1093]
---

## Goal
Evidence for whether Excel limits formulas typed into a cell, and defined-name bodies, to 8,192
characters, as it does for formulas stored in a file (T-1093, decision 0047).

## Context
Excel for the web refuses a workbook holding a stored formula longer than 8,192 characters (T-1091). The
xlsx importer now refuses the same workbooks. Nothing is probed yet for these cases:
- A formula set through the Graph API or typed in the UI. `range.formulas` with an 8,193 or 8,194
  character `=1+1+...` gives either an error or a value. The leading `=` may or may not count.
- A defined name whose body is longer than 8,192 characters, in a file and through the API.
- Characters outside the BMP. Excel may count UTF-16 code units, where the importer counts scalars.

Use the excel-probe skill. The oracle builds workbooks from files, so typed entry may need a manual Graph
call or desktop Excel. If only desktop Excel can answer, file the question in M14 (decision 0039).
Record the findings in docs/excel-semantics/syntax.md "Formulas over 8,192 characters refuse the
workbook". If they show an API limit, file an engine task to implement it.

## Acceptance criteria
- [ ] Each question above is answered with evidence or explicitly deferred to M14, and recorded in syntax.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing an API limit (file a task if the evidence calls for one).
