---
id: T-0156
title: "Array constants: reject sign forms Excel rejects ({++1}, {-+1}, {+'a'}, {+#N/A})"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Array constants accept exactly what Excel accepts: one optional sign attached to a number literal.

## Context
T-0147 found (docs/excel-semantics/syntax.md, "Array constants take a single optional sign") that Excel rejects `{++1}`, `{-+1}`, `{+-1}`, `{+"a"}`, `{+#N/A}`, `{- 1}`. The parser drops plus signs before `parse_array_item` (`expressions/parser/lists.rs`), so some of these are accepted. Verify each form first, then reject.

## Acceptance criteria
- [ ] Every rejected form in the semantics entry is a parse error; `{+1}`, `{-1}`, `{-1.5E1}` still parse (probe `array-constant-signs`).
- [ ] Unit tests cover each form; the semantics entry's "RustSheet today" line is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Unary signs outside array constants (T-0132).
