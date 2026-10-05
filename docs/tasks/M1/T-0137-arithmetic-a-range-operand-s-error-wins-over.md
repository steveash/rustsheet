---
id: T-0137
title: "Arithmetic: a range operand's error wins over the other operand's error"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
When the left operand of an arithmetic operator is a range (or array) holding an error and the
right operand is a scalar error, the result is the left operand's error, as in Excel:
`(Data!C1:C10 Data!B9:D9)/Data!D$5` with Data!C9 = `#N/A` and Data!D5 = `#DIV/0!` gives `#N/A`.

## Context
gen-m1 (oracle excel-graph, 2026-10-05): `gen-m1/4dc8ed1de05a61be` Calc!A250 (`intersection/arith`)
expects `#N/A`; the engine gives `#DIV/0!`. Found in T-0105: this is the only cell of that case the
intersection operator did not fix. The intersection gives the 1x1 range Data!C9, so plain
`=Data!C9:C9/Data!D$5` should show the same bug.
Semantics: docs/excel-semantics/coercion.md "An error in a range operand of arithmetic wins over
the right operand's error".

Root cause: `arithmetic.rs` `handle_arithmetic` turns the left range into an array
(`cast.rs` `get_number_or_array`) without looking at its elements, then returns the right operand's
error as soon as it sees it. Errors inside the left array only surface element by element in
`arithmetic_on_values`, which never runs. A scalar left error already wins (it returns first).
Unit test to add (it fails today; written during T-0105 and left out of that PR):
Data!C9 = `#N/A`, Data!D5 = `=1/0`; `=(Data!C1:C10 Data!B9:D9)/Data!D$5` and
`=Data!C9:C9/Data!D$5` both give `#N/A`.

Open question for the implementer: for a multi-cell left range (`=Data!C8:C9/Data!D5`) Excel
presumably gives an array whose elements pick the left error first and the right error otherwise.
There is no oracle evidence for that yet; probe it (`excel-probe` skill) before changing the
multi-cell case.

## Acceptance criteria
- [ ] `gen-m1/4dc8ed1de05a61be` Calc!A250 disappears from eval/baseline/gen-m1.txt, with no new
  F lines in any suite.
- [ ] Unit tests for the 1x1 cases above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Coercion of text elements in range arithmetic (T-0131).
