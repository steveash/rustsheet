---
id: T-1221
title: "Eval adapter: load stored:true error cells as stored constants, as xlsx import does"
milestone: M1
lane: harness
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
In `rustsheet-eval`'s engine adapter, a spec cell `{"t":"e","v":"#X","stored":true}` loads as the
error constant that xlsx import would give for `<c t="e"><v>#X</v></c>`. Today it loads as text
typed by a user. Without this, the frozen M1 probe `probes/error-stored-constants` cannot pass, even
after the engine has a `#BUSY!` kind (T-1222).

## Context
- spec2xlsx writes `"stored":true` errors as stored `t="e"` constants (T-0999). The adapter doesn't
  do the same. `CellInput::Typed(Value::Error(e))` in `crates/rustsheet-eval/src/case.rs` drops the
  `stored` key: `Value` is `#[serde(tag = "t", content = "v")]`, and the doc comment says "the engine
  adapter ignores it". `spec_input` in `crates/rustsheet-eval/src/engine.rs` then enters the error
  as `Input::Parse(e)`, i.e. as typed cell input. Typed input recognises only the formula-literal
  error names, so stored `#BUSY!` becomes the text "#BUSY!". That is the actual value in
  `make eval-show CASE=probes/error-stored-constants.case.json` (S!D1, S!I1 show `"#BUSY!"`, and
  S!G1 shows `TYPE` 2).
- xlsx import (`crates/rustsheet-xlsx/src/import/worksheets.rs`, the `t="e"` arms) uses the
  stored-name table `get_error_by_english_name` and falls back to `Error::ERROR` for an unknown
  name. The facade already reaches that table through `ErrorKind::from_literal`. A stored error
  constant can be entered as `Input::Value(Value::Error(XlError::new(kind)))`. The `Input::Value`
  arm in `crates/rustsheet/src/edit.rs` calls `update_cell_with_error`.
- Stored `#GETTING_DATA` and `#N/A` (rows 2 and 3 of the probe) already pass. They must keep
  passing.
- No Excel evidence is needed. The rule is to mirror xlsx import, which is what spec2xlsx makes
  Excel see.

## Acceptance criteria
- [ ] A spec error cell with `"stored": true` deserializes into a form the adapter can tell apart
      from a non-stored `{"t":"e"}` cell (for example a new `CellInput` variant placed before
      `Typed`). Non-stored error cells still load through `Input::Parse`, unchanged.
- [ ] The adapter enters a stored error through `Input::Value` with the kind
      `ErrorKind::from_literal(v)`. An unknown name falls back to `ErrorKind::Unknown` (`#ERROR!`),
      the same as the xlsx importer's `unwrap_or(Error::ERROR)`.
- [ ] Unit tests in `crates/rustsheet-eval` cover four cases. Stored `#N/A` loads as error `#N/A`.
      Stored `#GETTING_DATA` loads as `GettingData`. Stored `#BUSY!` loads as an error, not text
      (`#ERROR!` until T-1222 adds the kind). Non-stored `{"t":"e","v":"#N/A"}` is unchanged.
- [ ] The doc comment on `CellInput` and the "Cell inputs" description in
      `docs/architecture/eval-harness.md` (if it has one) say what `stored` now does.
- [ ] `make eval` shows no new F lines. Any `probes/error-stored-constants` cells that now pass
      (for example S!C1, E1, G1, which only need "an error that is not `#N/A`") are recorded with
      `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Adding a `#BUSY!` error kind (T-1222) and the SPEC-0003 decision on it (T-1020).
- spec2xlsx changes. It already writes stored constants.
