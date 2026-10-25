---
id: T-2627
title: "T of a cell holding text over 255 units is empty text; T of a long array element and REPT with an error count (T-2305 probes)"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Ours matches Excel on the three cells T-2305 found wrong: `T(cell)` of a cell holding text over 255 units, `T` of a long array element, and `REPT(long text, error)`.

## Context
Probes text-255-units-from-cell A4 and A12 (`T(D1)`, `T(D3)`: Excel "", ours `#VALUE!`), text-255-units-array-element A4 (`T` of a 256-unit element: Excel `#VALUE!`, ours the text) and text-255-units-rept-error-count A1 and A4 (`REPT(REPT("x",256),#N/A)` is `#N/A`, `#DIV/0!` likewise; ours `#VALUE!`). See docs/excel-semantics/text.md "REPT, CODE, UNICODE and T refuse a text argument over 255 UTF-16 units". Code: `reject_over_short_limit` (functions/text/common.rs) and its callers.

## Acceptance criteria
- [ ] The 5 known-failure lines for these probes disappear from eval/baseline/probes.txt (`make eval-baseline`), no new ones.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
