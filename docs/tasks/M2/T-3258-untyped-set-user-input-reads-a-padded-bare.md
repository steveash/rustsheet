---
id: T-3258
title: "Untyped set_user_input reads a padded bare column T[ Col ] as the engine's own text"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A padded bare column (`T[ Col ]`, `T[ Col]`, `T[Col ]`) entered from outside the engine through the untyped
`Model::set_user_input` (also the Python bindings' path) and in a defined-name body is read as Excel reads typed or
loaded text, while the engine's own re-parse of its printed `T[ Col ]` (after edits) keeps reading the column.

## Context
Follow-up from T-3069 (PR #2415 review). T-3069 added `Lexer::set_engine_text`, set in `Parser::parse` to
`!typed && !loaded`, so only typed and loaded text refuses a padded bare column. Untyped `set_user_input` is also the
path the engine's own print-then-parse callers use (`Model::extend_to`, `extend_copied_value`,
`move_cell_value_to_area`), so they need a separate marker before external untyped input can refuse it. Evidence:
probes/t2800-stored-ws-0..2 (stored: `#REF!`, FORMULATEXT `=SUM(#REF!)`), docs/excel-semantics/syntax.md
"Whitespace beside a missing argument". A defined name's body is unprobed: probe it first (excel-probe skill).

## Acceptance criteria
- [ ] Untyped external `set_user_input("=SUM(T[ Col ])")` matches Excel's typed result; the engine's own re-parse
      paths still read `T[ Col ]` as the column (tests for each).
- [ ] A defined-name body `T[ Col ]` is probed and matches Excel.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Printing changes for this-row references (T-3241).
