---
id: T-1145
title: "Workbook.set of XlError #CONNECT!/#BLOCKED!/#FIELD! stores text, not an error cell"
milestone: M1
lane: api
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Setting a cell to an error value through the typed facade (`Input::Value(Value::Error(..))`, Python `Workbook.set(r, XlError("#CONNECT!"))`) stores an error cell of that kind for every `ErrorKind`, so ERROR.TYPE of it is the kind's code.

## Context
Found in T-1014: `crates/rustsheet/src/edit.rs` (`Input::Value`, `Value::Error(e)`) calls
`model.set_user_input(.., e.kind.literal())`. The input parser treats `#CONNECT!`, `#BLOCKED!` and `#FIELD!` as text
(Excel rejects them as typed literals, syntax.md "Newer error literals"), so the cell becomes the text "#CONNECT!":
ERROR.TYPE of it is `#N/A` and ISERROR is FALSE, where `#SPILL!`/`#CALC!` constants give 9/14. A typed setter
should store the error itself (an `update_cell_with_error`-style engine call, not text parsing), like the
number/bool/text setters next to it. The typed input is an API value, not typed text, so it is not bound by
what Excel accepts in the formula bar.

## Acceptance criteria
- [ ] A facade test sets each `ErrorKind` (including `#CONNECT!`, `#BLOCKED!`, `#FIELD!`, `#GETTING_DATA`) as a
      typed value and reads it back as that error; ERROR.TYPE of each gives its code.
- [ ] The Python binding test covers `Workbook.set(r, XlError("#BLOCKED!"))`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Added `Model::update_cell_with_error` (engine) and used it for `Input::Value(Value::Error(..))`, so the typed setter stores an error cell of the given kind instead of parsing its literal as text.
- Facade test `typed_error_values_store_every_kind` (each Excel-coded kind, read back and ERROR.TYPE) and Python `test_set_stores_typed_error_cells` for `#GETTING_DATA`, `#CONNECT!`, `#BLOCKED!`, `#FIELD!`.
- Eval delta: none.
