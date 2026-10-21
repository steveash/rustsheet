---
id: T-2323
title: "Sheet ops delete_row, insert_column, delete_column, delete_sheet and table convert/delete in the oracle and engine adapter"
milestone: M8
lane: harness
status: done
priority: P2
size: M
tier: sonnet
depends_on: []
---

## Goal
A probe case can delete rows, insert and delete columns, delete a sheet, and convert a table to a range or
delete it, and both excel-graph and the engine adapter replay the edit.

## Context
T-2197 could only probe row inserts: the oracle replays `add`, `rename`, `move` and `insert_row`
(`_replay_sheet_ops` in tools/src/rustsheet_tools/oracle/backends.py; spec2xlsx `SHEET_OPS`; `SheetOp`
in crates/rustsheet-eval/src/case.rs, `apply_sheet_ops` in engine.rs). Add, following `insert_row`:
`delete_row` (`range('r:r')/delete` shift Up), `insert_column` / `delete_column` (`range('C:C')/insert|delete`
shift Right/Left; spec key `column`, a letter), `delete_sheet` (`DELETE worksheets('S')`),
`convert_table` (`POST tables('T')/convertToRange`) and `delete_table` (`DELETE tables('T')`). The engine
side uses `delete_rows`, `insert_columns`, `delete_columns`, `delete_sheet` (decision 0076 harness edits)
and `remove_table`; the engine has no convert-to-range, so that op is refused by the adapter (the case
then scores as an adapter error) until an engine task adds it: say so in case.rs. Each op logs a
`SHEET OP` line as today. Cut and paste, row/column moves, undo and sheet duplication have no Graph REST
endpoint and stay out (desktop questions, decision 0039).

## Acceptance criteria
- [ ] spec2xlsx, the Graph backend and the engine adapter accept the new ops, with tests for each
  (tools pytest, engine.rs unit tests), and refuse malformed ones.
- [ ] docs/architecture/oracle.md and the spec2xlsx docstring list the ops.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probing Excel with them (T-2324).

## Outcome
spec2xlsx, the Graph backend (`_RANGE_EDITS` and the sheet/table calls) and the engine adapter accept
`delete_row`, `insert_column`, `delete_column`, `delete_sheet`, `convert_table` and `delete_table`, with
tools pytest and engine/eval unit tests, and refuse malformed ones. The facade gained harness-gated
`delete_rows`, `insert_columns`, `delete_columns` and `remove_table` (decision 0098; `Txn::delete_sheet`
already existed). The adapter refuses `convert_table` (no engine convert-to-range). Docs: oracle.md,
eval-harness.md, spec2xlsx docstring. No eval cases changed; eval baseline unchanged.
