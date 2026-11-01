---
id: T-1683
title: "FORMULATEXT and unterminated structured refs: keep structured-reference text; reject t[@Qty without closing bracket"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
FORMULATEXT of a formula with a structured reference shows what Excel shows, and an unterminated structured reference (`=T[@Qty`, `=T[Col`) is a parse error.

## Context
- Since T-2195 every structured reference except `#This Row` is kept as written, and FORMULATEXT shows the canonical long form the storage printer writes (`=SUM(Sales[Qty])`, the table name in its typed case, `Sales[]` as bare `Sales`); `#This Row` is still lowered, so FORMULATEXT shows `=$A$2` for `=T[@Qty]` until T-1699. The probe should also cover the case of column names as typed (`sales[qty]`), whether Excel drops a redundant `[#Data]`, and the `T[[Col]]` → `T[Col]` spelling. Excel's FORMULATEXT output is unprobed: use the `excel-probe` skill first.
- `consume_escaped_until` (lexer/structured_references.rs) accepts input with no closing `]` and moves the position past the end; found in T-1672 review. Confirm Excel rejects it by probe.

- Evidence for the `#This Row` spelling (excel-graph, 2026-10-28, T-2845; references.md "Structured references when
  the table changes"): FORMULATEXT prints `=Sales[@Qty]` for a stored `Sales[[#This Row],[Qty]]`, and `=[@Qty]*2` for
  `=Sales[@Qty]*2` inside table Sales. Ours prints the long form with the name. Cases: probes/table-edits
  table-insert-row-this-row-above-header, -this-row-inside-data, -calc-column-above-header, -calc-column-inside-data
  (the FORMULATEXT cells; their #This Row values are T-3037's).

## Acceptance criteria
- [ ] Probe Excel FORMULATEXT for `T[@Qty]`, `T[[#This Row],[Qty]]`, `T[Qty]`; record in docs/excel-semantics/references.md.
- [ ] Include a bare table name (`ROWS(Sales)`), now lowered to a range like `Sales[]`.
- [ ] Implement the probed behavior; unterminated references give a lexer error.
- [ ] Probe whether Excel rejects the unbracketed multi-word `T[@Unit Price]` (the lexer accepts it now); reject if so.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rows outside the table (T-1675).
