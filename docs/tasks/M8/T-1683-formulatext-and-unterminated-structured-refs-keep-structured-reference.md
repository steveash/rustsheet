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
- The parser lowers structured references to cell references (`structured_reference_operand`, parser/operands.rs), so FORMULATEXT shows `=$A$2` for `=T[@Qty]`. Excel's FORMULATEXT output is unprobed: use the `excel-probe` skill first.
- `consume_escaped_until` (lexer/structured_references.rs) accepts input with no closing `]` and moves the position past the end; found in T-1672 review. Confirm Excel rejects it by probe.

## Acceptance criteria
- [ ] Probe Excel FORMULATEXT for `T[@Qty]`, `T[[#This Row],[Qty]]`, `T[Qty]`; record in docs/excel-semantics/references.md.
- [ ] Implement the probed behavior; unterminated references give a lexer error.
- [ ] Probe whether Excel rejects the unbracketed multi-word `T[@Unit Price]` (the lexer accepts it now); reject if so.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rows outside the table (T-1675).
