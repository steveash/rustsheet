---
id: T-3307
title: "Probe how a U+3000 intersection formula's references move when rows and columns are inserted or deleted before them"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for what happens to `=A1:A3<U+3000>A2:B2` (which Excel evaluates to `#NAME?`) when a
row or column is inserted or deleted before its references: whether FORMULATEXT shows shifted
references (so Excel still parses them as references) or the text unchanged (so the whole span is one
name), and where that span begins and ends. references.md records the answer.

## Context
- references.md "Whitespace in the intersection operator" (T-0138, T-0148): "The engine turns a U+3000
  intersection into one undefined name holding its text; where Excel's name begins and ends, and so how
  references inside it update on row/column inserts, is UNVERIFIED." Evidence today:
  eval/suites/probes/cases/intersection-whitespace.case.json H7 (value only, no structural edit).
- The case format and both backends replay structural edits (`sheet_ops`: `insert_row`, `delete_row`,
  `insert_column`, `delete_column`; tools/src/rustsheet_tools/sheetop.py, T-1608, T-1933, T-2323). A
  layout to copy: eval/suites/probes/cases/sheet-ops/sheet-ops-insert-row-legacy-cell.case.json.
- Shapes, each in its own formula cell with a FORMULATEXT readout of it, after each of: insert row 1,
  insert column A, delete row 1 (with the references below it), and no edit as the control:
  `=A2:A4<U+3000>A3:B3`, `=A2<U+3000>B2`, `=SUM(A2:A4<U+3000>A3:B3)`, `=A2:A4<U+3000>A3:B3+B2`
  (a plain reference after the span) and `=B2+A2:A4<U+3000>A3:B3` (one before it).
- Write U+3000 in the stored formula as T-0148 did (excel-graph loads it; CR, TAB and NBSP there are
  refused, so do not use them).

## Acceptance criteria
- [ ] Probe case `probes/t3307-ideographic-space-intersection-edits` (milestone M2) committed with its
      excel-graph sidecar: the five shapes under the four edits, value and FORMULATEXT each.
- [ ] references.md replaces the UNVERIFIED sentence quoted above with the observed rule, citing case
      and cells.
- [ ] Every cell passes, or the failing cells are known failures recorded by `make eval-baseline`
      (0 regressions) and an engine task is filed that lists them (case stem and A1 names in one
      bullet, decision 0159).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the parser or the reference-move code (the follow-up task).
- Parenthesized U+3000 operands (T-3284); other separators.
