---
id: T-0263
title: "The & operator broadcasts a scalar error operand against an array (leftmost error per element)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`array & scalar_error` and `scalar_error & array` return an array of the array's size whose every
element is the leftmost error, as the arithmetic operators already do (docs/excel-semantics/arrays.md
"A scalar error operand broadcasts against an array operand"). Today `&` returns the scalar error
whenever either operand is one, so `MID({"a","b"},0,1)&NA()` is `#N/A` where Excel gives
`{#VALUE!,#VALUE!}`.

## Context
Found by T-0212 (decision 0037): `gen-m2/1b6ceed7489e0800` Calc!A332
`=(MID(INDEX(Data!A1:B6,2,0),(Data!$D1+Data!E7),SUM(...))&VLOOKUP(UPPER(" x "),Data!B3:E7,1,Data!$D7))`
is `#VALUE!` in Excel (excel-graph): the MID broadcast over the row is all `#VALUE!` (start < 1) and the
VLOOKUP is `#N/A`. Ours is `#N/A`. The cell passed before T-0212 only because the old VLOOKUP found a value.
Code: `handle_concatenate` in `crates/rustsheet-engine/src/arithmetic.rs` returns early when
`get_string_or_array` fails on either operand; `handle_arithmetic` in the same file already broadcasts a
scalar error (T-0209), so reuse that path rather than writing a second one (CLAUDE.md directive 3).
Probe first (excel-probe skill): `{1,2}&NA()`, `NA()&{1,2}`, `INDEX({1;#DIV/0!}&NA(),2)`,
`INDEX(NA()&{1;#DIV/0!},2)`, `ROWS(NA()&{1;2;3})`, a multi-cell range against a scalar error, and two
scalar errors (`(1/0)&NA()` stays `#DIV/0!`).

## Acceptance criteria
- [ ] Probe case filled by the oracle; docs/excel-semantics/arrays.md entry extended to `&` with it.
- [ ] `gen-m2/1b6ceed7489e0800.case.json` Calc!A332 disappears from eval/baseline/gen-m2.txt with no
      new F lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Comparison operators (`=`, `<`, ...) unless the probe shows they share the bug: file a task.
